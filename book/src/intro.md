![blue haired anime girl forging a letter](./banner.png)

# ✍️ Forgery

Forgery is a Solidity web-server runtime written in Rust and based on the `foundry-rs`
stack by Paradigm. It implements a fully-featured HTTP framework alongside
Foundry's suite of tools and cheatcodes. If you are unfamiliar with Foundry,
it is highly recommended to read up on it, and specifically
[Forge](https://book.getfoundry.sh/forge/), Foundry's scripting environment.

## 🤔 Wtf
Write Solidity code as your backend, while running on a forked Foundry
environment. This lets you interact with contracts directly, efficiently
retrieve information onchain and use Foundry's cheatcodes to simulate
transactions, mock internal calls and more.

Web3 backends today are usually written in languages not fit for smart contract
interactions. JSON ABIs and awkward async APIs encourage convoluted code with
inconsistent behavior. Instead, why not write Solidity directly, use Solidity
interfaces and execute in a completely synchronous way, *excatly how it would
behave onchain*.

For example, a Uniswap price endpoint would be trivially implemented in
Forgery:

```solidity
contract Index is Server {
    using JSONBodyParser for Request;
    using JSONBodyWriter for Response;

    QuoterV2 quoter = QuoterV2(0x61fFE014bA17989E743c5F6cB21bF9697530B21e);

    function start () external override {
        router.post('/quote', quote);
    }

    function quote (
        Request calldata request
    ) public {
        address[] memory tokens = request.json().at('tokens').asAddressArray();
        uint amountIn = request.json().at('amountIn').asUint();
        uint fee = request.json().at('fee').asUint();

        (uint amountOut,,,) = quoter.quoteExactInputSingle(
            QuoterV2.QuoteExactInputSingleParams({
                tokenIn: tokens[0],
                tokenOut: tokens[1],
                amountIn: amountIn,
                fee: uint24(fee),
                sqrtPriceLimitX96: 0
            })
        );

        response.status = 200;
        response.header('content-type', 'application/json');
        response.write('amountOut', amountOut);
    }
}
```

Anyone who has tried implementing the same feature using Node.js, Python or Rust,
knows how awkward and involved it would be.

Forgery is the native web3 backend.

## 🔨 How it works
Under the hood Forgery is based on [Foundry](https://book.getfoundry.sh/) and
[Hyper](https://hyper.rs/). Together a new Solidity runtime is born, one that
merges web2 and web3 seamlessly.

Forgery sets up a low overhead Hyper instance while running a Foundry EVM
instance on the main thread. Your contracts are deployed on the Foundry EVM
instance and the Hyper server communicates with your contracts by broadcasting
transations.

Every request is converted into ABI-encoded transaction using
[Alloy](https://github.com/alloy-rs/core), which is then passed into the
Solidity runtime as calldata. The contract responds with an ABI-encoded response
which is converted into an HTTP response that is then sent back to the user.

In addition, [Forgery SDK](./forgery-sdk.md) is a framework which lets you write
web-servers intuitively in a familiar way akin to other popular web frameworks.
