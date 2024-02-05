# Forgery SDK
The Forgery SDK is meant to be a fully-fledged web framework for Forgery.

Its use is *optional*, though highly recommended.

It introduces a simple API that would be somewhat familiar for those coming with
a background in other web frameworks.

## Overview
The main entry point implements an abstract `Server` contract. The `Server`
contract utilizes a `Router` to direct requests to their endpoint
implementations.
The `Request` object is kept in the calldata to reduce overhead, while the
`Response` object is copied into contract storage for easier manipulation.
In addition, Forgery SDK provides two utilities for working with JSONs:
`JSONBodyParser` and `JSONBodyWriter`. These are meant to help parse & generate
JSON objects from the `Request` and `Response` objects respectively.
Other than that, the `Server` contract exposes the Foundry cheatcodes via `vm`,
similar to Forge scripts or tests.

## Hello, world
A basic example of a Forgery SDK contract might look like so:
```solidity
contract Index is Server {
    function start () external override {
        router.get('/hello', hello);
    }

    function hello (
        Request calldata request
    ) internal {
        response.status = 200;
        response.header('content-type', 'text/plain');
        response.body = 'Hello, world';
    }
}
```
