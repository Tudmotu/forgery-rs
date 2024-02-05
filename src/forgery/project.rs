use std::collections::{BTreeMap, HashSet};

use alloy_primitives::{Address, Bytes as EvmBytes, U256};
use alloy_sol_types::SolCall;
use eyre::{Context, ContextCompat, ErrReport, Result};
use forge::{
    backend::{Backend, DatabaseExt},
    executors::{DeployResult, Executor, ExecutorBuilder},
    fork::CreateFork,
    inspectors::CheatsConfig,
    link::{link_with_nonce_or_address, PostLinkInput, ResolvedDependency},
    opts::EvmOpts,
    revm::primitives::{db::DatabaseCommit, Env as RevmEnv, SpecId},
};
use foundry_common::{
    compact_to_contract,
    compile::{self, ContractSources},
    fs::{self, canonicalize_path},
};
use foundry_compilers::{
    artifacts::{CompactContractBytecode, ContractBytecode, ContractBytecodeSome, Libraries},
    contracts::ArtifactContracts,
    ArtifactId, Project,
};
use foundry_config::Config;

use crate::forgery::types::startCall;

pub async fn executor(opts: EvmOpts, revm_env: RevmEnv) -> Result<Executor, ErrReport> {
    // The db backend that serves all the data.
    let fork_url = opts.fork_url.clone().expect("DB fork url is missing");
    let fork = CreateFork {
        url: fork_url,
        enable_caching: true,
        env: revm_env.clone(),
        evm_opts: opts.clone(),
    };
    let db = Backend::spawn(Some(fork)).await;
    let config = Config::load();

    let builder = ExecutorBuilder::new()
        .inspectors(|stack| {
            stack
                .debug(true)
                .cheatcodes(CheatsConfig::new(&config, opts.clone()).into())
                .trace(true)
        })
        .spec(SpecId::CANCUN)
        .gas_limit(opts.gas_limit());

    Ok(builder.build(revm_env, db))
}

pub fn build() -> Result<BuildOutput, ErrReport> {
    let config = Config::load();
    let project = config.project().unwrap();

    let index_path = "./src/Index.sol";

    let target_result = canonicalize_path(index_path);

    if let Err(err) = target_result {
        return Err(eyre::eyre!("No index contract found: {err}"));
    }

    let target_contract = target_result.unwrap();

    let output =
        compile::compile_target_with_filter(&target_contract, &project, false, false, Vec::new())?;

    let output = output.with_stripped_file_prefixes(project.root());

    let mut sources: ContractSources = Default::default();

    let contracts = output
        .into_artifacts()
        .map(|(id, artifact)| -> Result<_> {
            // Sources are only required for the debugger, but it *might* mean that there's
            // something wrong with the build and/or artifacts.
            if let Some(source) = artifact.source_file() {
                let path = source
                    .ast
                    .ok_or_else(|| eyre::eyre!("source from artifact has no AST"))?
                    .absolute_path;
                let abs_path = project.root().join(path);
                let source_code = fs::read_to_string(abs_path).wrap_err_with(|| {
                    format!(
                        "failed to read artifact source file for `{}`",
                        id.identifier()
                    )
                })?;
                let contract = artifact.clone().into_contract_bytecode();
                let source_contract = compact_to_contract(contract)?;
                sources
                    .0
                    .entry(id.clone().name)
                    .or_default()
                    .insert(source.id, (source_code, source_contract));
            } else {
                println!("Source not found");
            }
            Ok((id, artifact))
        })
        .collect::<Result<ArtifactContracts>>()?;

    let mut run_dependencies = vec![];
    let mut contract = CompactContractBytecode::default();
    let mut highlevel_known_contracts = BTreeMap::new();

    let target_fname = canonicalize_path(index_path)
        .wrap_err("Couldn't convert contract path to absolute path.")?
        .strip_prefix(project.root())
        .wrap_err("Couldn't strip project root from contract path.")?
        .to_str()
        .wrap_err("Bad path to string.")?
        .to_string();

    let no_target_name = true;

    let mut extra_info = ExtraLinkingInfo {
        no_target_name,
        target_fname: target_fname.clone(),
        contract: &mut contract,
        dependencies: &mut run_dependencies,
        matched: false,
        target_id: None,
    };

    // link_with_nonce_or_address expects absolute paths
    let libraries_addresses = config.parsed_libraries().unwrap();
    let mut libs = libraries_addresses.clone();
    for (file, libraries) in libraries_addresses.libs.iter() {
        if file.is_relative() {
            let mut absolute_path = project.root().clone();
            absolute_path.push(file);
            libs.libs.insert(absolute_path, libraries.clone());
        }
    }

    link_with_nonce_or_address(
        contracts.clone(),
        &mut highlevel_known_contracts,
        libs,
        Address::ZERO,
        0u64,
        &mut extra_info,
        |post_link_input| {
            let PostLinkInput {
                contract,
                known_contracts: highlevel_known_contracts,
                id,
                extra,
                dependencies,
            } = post_link_input;

            fn unique_deps(deps: Vec<ResolvedDependency>) -> Vec<(String, EvmBytes)> {
                let mut filtered = Vec::new();
                let mut seen = HashSet::new();
                for dep in deps {
                    if !seen.insert(dep.id.clone()) {
                        continue;
                    }
                    filtered.push((dep.id, dep.bytecode));
                }

                filtered
            }

            // if it's the target contract, grab the info
            if extra.no_target_name {
                // Match artifact source, and ignore interfaces
                if id.source == std::path::Path::new(&extra.target_fname)
                    && contract
                        .bytecode
                        .as_ref()
                        .map_or(false, |b| b.object.bytes_len() > 0)
                {
                    if extra.matched {
                        eyre::bail!("Multiple contracts in the target path. Please specify the contract name with `--tc ContractName`")
                    }
                    *extra.dependencies = unique_deps(dependencies);
                    *extra.contract = contract.clone();
                    extra.matched = true;
                    extra.target_id = Some(id.clone());
                }
            } else {
                let (path, name) = extra
                    .target_fname
                    .rsplit_once(':')
                    .expect("The target specifier is malformed.");
                let path = std::path::Path::new(path);
                if path == id.source && name == id.name {
                    *extra.dependencies = unique_deps(dependencies);
                    *extra.contract = contract.clone();
                    extra.matched = true;
                    extra.target_id = Some(id.clone());
                }
            }

            if let Ok(tc) = ContractBytecode::from(contract).try_into() {
                highlevel_known_contracts.insert(id, tc);
            }

            Ok(())
        },
        project.root(),
    )?;

    let target = extra_info
        .target_id
        .ok_or_else(|| eyre::eyre!("Could not find target contract: {}", target_fname))?;

    let (new_libraries, predeploy_libraries): (Vec<_>, Vec<_>) =
        run_dependencies.into_iter().unzip();

    // Merge with user provided libraries
    let mut new_libraries = Libraries::parse(&new_libraries)?;
    for (file, libraries) in libraries_addresses.libs.into_iter() {
        new_libraries
            .libs
            .entry(file)
            .or_default()
            .extend(libraries)
    }

    Ok(BuildOutput {
        target,
        contract,
        known_contracts: contracts,
        highlevel_known_contracts: ArtifactContracts(highlevel_known_contracts),
        predeploy_libraries,
        sources: Default::default(),
        project,
        libraries: new_libraries,
    })
}

pub fn deploy(executor: &mut Executor, build: BuildOutput) -> Result<Address, ErrReport> {
    let CompactContractBytecode { bytecode, .. } = build.contract;
    executor.set_nonce(Address::ZERO, 0)?;

    // We max out their balance so that they can deploy and make calls.
    executor.set_balance(Address::ZERO, U256::MAX)?;

    // Deploy libraries
    build.predeploy_libraries.iter().for_each(|code| {
        executor
            .deploy(Address::ZERO, code.clone(), U256::ZERO, None)
            .expect("couldn't deploy library");
    });

    let address = Address::ZERO.create(executor.get_nonce(Address::ZERO)?);

    // Set the contracts initial balance before deployment, so it is available during the
    // construction
    executor.set_balance(address, U256::MAX)?;

    // Deploy an instance of the contract
    let DeployResult { address, .. } = executor
        .deploy(
            Address::ZERO,
            bytecode.expect("No bytecode").into_bytes().unwrap(),
            U256::ZERO,
            None,
        )
        .map_err(|err| eyre::eyre!("Failed to deploy script:\n{}", err))?;

    executor.backend.add_persistent_account(address);

    let fn_call = startCall {};
    let calldata = fn_call.abi_encode();
    let call = executor.call_raw(Address::ZERO, address, calldata.into(), U256::ZERO);
    let res = call
        .map_err(|err| {
            panic!("Error occured while trying to execute start(): {}", err);
        })
        .unwrap();

    if res.reverted {
        panic!("start() call reverted with: {:#?}", res.exit_reason);
    }

    if let Some(changes) = &res.state_changeset {
        executor.backend.commit(changes.clone());
    }
    Ok(address)
}

struct ExtraLinkingInfo<'a> {
    no_target_name: bool,
    target_fname: String,
    contract: &'a mut CompactContractBytecode,
    dependencies: &'a mut Vec<(String, EvmBytes)>,
    matched: bool,
    target_id: Option<ArtifactId>,
}

pub struct BuildOutput {
    pub project: Project,
    pub target: ArtifactId,
    pub contract: CompactContractBytecode,
    pub known_contracts: ArtifactContracts,
    pub highlevel_known_contracts: ArtifactContracts<ContractBytecodeSome>,
    pub libraries: Libraries,
    pub predeploy_libraries: Vec<EvmBytes>,
    pub sources: ContractSources,
}
