# Quickstart
To start writing Forgery backends, you will first need to [install Forgery](./installation.md).
If you are following the [Core API](./intro/core-api.md), all you need is to
implement the `src/Index.sol` contract. This is not recommended ― the preferred
method is using a framework such as [Forgery SDK](./forgery-sdk.md).

Otherwise, it is recommended to use the [Forgery boilerplate](https://github.com/Tudmotu/forgery-boilerplate).
The boilerplate includes a simple example contract with tests, which should get
you up an running in no time.

Technically speaking, a Forgery project is simply a Foundry Forge project. This
means you can configure it using `foundry.toml` and install dependencies using
`forge install`.

The `forgery` command includes a utility for generating a basic example project
based on the `forgery-boilerplate` repo. This utility requires `forge` to be
installed. Follow the [Foundry docs](https://book.getfoundry.sh/getting-started/installation)
for instructions. Once cloned, treat the Forgery project like any other Forge project.
```console
forgery init
```
This command will generate the project inside the current working directory,
similar to `forge init`.

Make sure to create a `.env` file with `FORGERY_RPC` configured.

Now all that is left is to start modifying `Index.sol` to implement the desired
logic.
