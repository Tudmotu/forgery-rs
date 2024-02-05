use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use std::{convert::Infallible, thread};

use alloy_primitives::{Address, U256};
use alloy_providers::provider::{Provider, TempProvider};
use alloy_sol_types::{SolCall, SolValue};
use dotenv::dotenv;
use eyre::Result;
use forge::revm::precompile::{Precompiles, SpecId as PrecompileSpec};
use forge::revm::primitives::SpecId;
use forge::revm::JournaledState;
use forge::{
    decode::decode_console_logs,
    executors::Executor,
    opts::Env,
    opts::EvmOpts,
    revm::primitives::{db::DatabaseCommit, BlobExcessGasAndPrice, BlockEnv, Env as RevmEnv},
};
use foundry_evm_core::backend::DatabaseExt;
use http_body_util::Full;
use hyper::body::Bytes as HyperBytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{body::Incoming, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::forgery::project;
use crate::forgery::types::{serveCall, SolHttpRequest, SolHttpResponse};
pub mod cmd;
pub mod forgery;

async fn forgery(
    executor_arc: Arc<Mutex<Executor>>,
    index_addr: Address,
    req: Request<Incoming>,
) -> Result<Response<Full<HyperBytes>>, Infallible> {
    let mut executor = executor_arc.lock().await;
    let fn_call = match SolHttpRequest::from_incoming(req).await {
        Ok(req_struct) => serveCall { _0: req_struct },
        Err(err) => {
            println!("{}", err);
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(HyperBytes::from("Request parsing failed")))
                .unwrap());
        }
    };
    let calldata = fn_call.abi_encode();
    let call = executor.call_raw(Address::ZERO, index_addr, calldata.into(), U256::ZERO);
    match call {
        Ok(res) => {
            if let Some(changes) = &res.state_changeset {
                executor.backend.commit(changes.clone());
            }

            let console_logs = decode_console_logs(&res.logs);
            if !console_logs.is_empty() {
                for log in console_logs {
                    println!("{}", log);
                }
            }

            if !res.reverted {
                match SolHttpResponse::abi_decode(&res.result, true) {
                    Ok(value) => Ok(value.into()),
                    Err(err) => {
                        println!("Error parsing response from contract: {}", err);
                        Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Full::new(HyperBytes::from("Response parsing failed")))
                            .unwrap())
                    }
                }
            } else {
                let reason = res.exit_reason;
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Full::new(HyperBytes::from(format!(
                        "Request reverted: {reason:#?}"
                    ))))
                    .unwrap())
            }
        }
        Err(err) => {
            println!("{}", err);
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(HyperBytes::from("Forgery encountered an error")))
                .unwrap())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "init" {
        cmd::init(args);
        return Ok(());
    }

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    let rpc_url = std::env::var("FORGERY_RPC").expect("FORGERY_RPC must be set");

    let env = Env {
        gas_limit: u64::MAX,
        chain_id: None,
        gas_price: Some(0u64),
        block_base_fee_per_gas: 0,
        tx_origin: Address::ZERO,
        block_coinbase: Address::ZERO,
        block_timestamp: 0u64,
        block_number: 0u64,
        block_difficulty: 0u64,
        block_prevrandao: U256::MAX.into(),
        block_gas_limit: Some(u64::MAX),
        code_size_limit: Some(usize::MAX),
    };

    let opts = EvmOpts {
        env: env.clone(),
        fork_url: Some(rpc_url.clone()),
        fork_block_number: None,
        fork_retries: Some(5),
        fork_retry_backoff: None,
        compute_units_per_second: None,
        no_rpc_rate_limit: true,
        no_storage_caching: false,
        initial_balance: U256::from(0),
        sender: Address::ZERO,
        ffi: false,
        verbosity: 1u8,
        memory_limit: u64::MAX,
    };

    let mut revm_env = RevmEnv {
        block: BlockEnv {
            basefee: U256::from(env.block_base_fee_per_gas),
            coinbase: env.block_coinbase,
            difficulty: U256::from(env.block_difficulty),
            gas_limit: U256::from(env.block_gas_limit.unwrap()),
            number: U256::from(env.block_number),
            prevrandao: Some(env.block_prevrandao),
            timestamp: U256::from(env.block_timestamp),
            blob_excess_gas_and_price: Some(BlobExcessGasAndPrice {
                excess_blob_gas: 0u64,
                blob_gasprice: 0u128,
            }),
        },
        cfg: Default::default(),
        tx: Default::default(),
    };

    let mut executor = project::executor(opts.clone(), revm_env.clone())
        .await
        .expect("Failed to create EVM executor");
    let build_result = project::build().expect("Project build failed");
    let address = project::deploy(&mut executor, build_result).expect("Failed to deploy project");

    println!("... done!");
    println!("Listening on port: {}", 3000);

    let executor_mutex = Arc::new(Mutex::new(executor));
    let executor_mutex_clone = executor_mutex.clone();
    tokio::task::spawn(async move {
        let executor_mutex = executor_mutex_clone.clone();
        let opts = opts.clone();
        loop {
            thread::sleep(Duration::from_secs(1));
            let mut executor = executor_mutex.lock().await;
            let fork_id = executor.backend.active_fork_id();
            let mut journaled_state = JournaledState::new(
                SpecId::CANCUN,
                Precompiles::new(PrecompileSpec::CANCUN)
                    .addresses()
                    .into_iter()
                    .copied()
                    .collect(),
            );
            let fork_url = opts.clone().fork_url.clone().unwrap();
            match Provider::try_from(&fork_url) {
                Ok(provider) => match provider.get_block_number().await {
                    Ok(block_number) => {
                        let _ = executor.backend.roll_fork(
                            fork_id,
                            U256::from(block_number),
                            &mut revm_env,
                            &mut journaled_state,
                        );
                    }
                    Err(e) => println!("Error getting latest block: {}", e),
                },
                Err(e) => println!("Error getting provider: {}", e),
            }
        }
    });

    loop {
        let executor_mutex = executor_mutex.clone();
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(|req| forgery(executor_mutex.clone(), address, req)),
                )
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
