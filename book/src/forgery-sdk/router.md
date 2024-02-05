# Router
The `Router` object lets you map between routes and functions.
It is a user-defined Solidity type which saves a mapping of paths to functions
per HTTP method.

The `Router` object expects your routes to comply with the following signature:
```solidity
function routeName (Request calldata) external;
```

The router does not execute routes on its own. It is only used as an abstraction
over a nested struct datastructure. Its purpose is convenience. The actual
executor is the `Server` contract.

The `Router` API exposes functions to register routes for each HTTP method. You
would usually register the routes in the `start()` function.

## Example
```solidity
import 'forgery-sdk/Server.sol';

contract Index is Server {
    function start () external override {
        router.get('/query', queryOrders);
        router.post('/create', createOrder);
        router.put('/replace', replaceOrder);
        router.patch('/update', updateOrder);
        router.del('/delete', deleteOrder);
    }
}
```
