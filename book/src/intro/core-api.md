# Core API
The Forgery Core API is a simple Solidity interface that must be implemented in
order for Forgery to communicate properly with your backend.

<div class="warning">
For most use cases it is probably best to use a framework such as <a href="../forgery-sdk.html">Forgery SDK</a>
instead of directly interfacing with the Forgery Core API.<br>
The API is documented
here for posterity but is not intended to be used directly.
</div>


## Webserver interface
Forgery requires you to implement the following:
1. You entrypoint contract must be located at `./src/Index.sol`
2. The contract must implement a `start()` function
3. The contract must implement a `server(Request calldata) returns (Response memory)` function

Here is the expected interface:
```solidity
interface ForgeryServer {
    struct SolHttpHeader {
        string key;
        string value;
    }

    struct SolHttpRequest {
        string method;
        string uri;
        SolHttpHeader[] headers;
        bytes body;
    }

    struct SolHttpResponse {
        uint16 status;
        SolHttpHeader[] headers;
        bytes body;
    }

    function start () external;
    function serve (SolHttpRequest calldata) external returns (SolHttpResponse memory);
}
```

### `start()`
This method is called on deployment. Whenever you start up your server, your
contract is deployed on the Foundry instance. This method will run *once*, right
after the server starts up, but before it starts listening on any connections.

### `serve()`
This method will be executed for *every* incoming request. This is the main
entrypoint into your backend. It is usually recommended to use some sort of
router helper to help manage different endpoint, such as the one available in
the [Forgery SDK](../forgery-sdk.md).
