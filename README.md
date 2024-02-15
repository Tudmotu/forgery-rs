<p style="text-align:center">
    <img src="./book/src/banner.png" width="100%" />
</p>

# ✍️ Forgery

Forgery is a Solidity web-server runtime written in Rust and based on the `foundry-rs`
stack by Paradigm. It implements a fully-featured HTTP framework alongside
Foundry's suite of tools and cheatcodes. If you are unfamiliar with Foundry,
it is highly recommended to read up on it, and specifically
[Forge](https://book.getfoundry.sh/forge/), Foundry's scripting environment.

## 📚 Documentation
See the [Forgery docs](https://tudmotu.github.io/forgery-rs/) for more comprehensive overview of Forgery internals.

## 🏃 Quickstart
This assumes you have Rust, Cargo and Foundry installed. Follow the
[Foundry docs](https://book.getfoundry.sh/getting-started/installation) for
installation instructions.
Compilation might take a few minutes depending on your machine.


Install using the script (see requirements below):
```console
curl -L https://raw.githubusercontent.com/Tudmotu/forgery-rs/main/getforgery.sh | bash
```

Create a directory for your project and change to it:
```console
mkdir myproject && cd myproject
```

Generate the Forgery example project:
```console
forgery init
```

Configure your RPC provider (required):
```console
echo 'FORGERY_RPC=<YOUR_RPC>' > .env
```

Run the server:
```console
forgery
```

Should show the following output:
```
[⠰] Compiling...
[⠆] Compiling 30 files with 0.8.24
[⠊] Solc 0.8.24 finished in 4.81s
Compiler run successful!
... done!
Listening on port: 3000
```

You can test it out using `curl`:
```console
curl http://localhost:3000/
```

You should get this response:
```
Hello, world!
```

Technically speaking, a Forgery project is simply a Foundry Forge project. This
means you can configure it using `foundry.toml` and install dependencies using
`forge install`.

## 📦 Installation
Installation is done from source currently. We provide a helper script that will
install Forgery using cargo by cloning the repo and compiling locally. It might
take a couple of minutes to complete.

### Requirements
You will need the following tools installed:
- bash
- curl
- git
- Rust + Cargo
- Foundry

### Installation
#### Helper
Use the helper script:
```console
curl -L https://raw.githubusercontent.com/Tudmotu/forgery-rs/main/getforgery.sh | bash
```

#### Manually
Clone the git repo:
```console
git clone https://github.com/Tudmotu/forgery-rs.git
```

Inside the repo, install using Cargo:
```console
cargo install --locked --path .
```

Optionally, add the Cargo binary directory to your `$PATH` if it's not already
included:
```console
echo 'export PATH=$PATH:~/.cargo/bin' >> ~/.zshrc
```
<div class="warning">
NOTE:<br>
This command might vary depending on your OS and shell. This instruction
assumes a POSIX OS with zsh.
</div>

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
    ) internal {
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
