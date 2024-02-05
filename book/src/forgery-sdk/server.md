# Server contract
The `Server` contract implements the basic structure of a Forgery app.
It instantiates a `Router` for you which lets you easily map paths to functions.
The `Router` expects functions with `(Request calldata)` signatures.

To use the `Server` contract, import it and inherit from it:
```solidity
import 'forgery-sdk/Server.sol';

contract Index is Server {
    function start () external override {
        router.get('/hello', hello);
    }

    function hello (
        Request calldata request
    ) public {
        // ...
    }
}
```

The `start()` function is executed when the server starts up. This is where you
would usually register your routes using the `Router`.
