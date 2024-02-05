# `JSONBodyWriter`
The `JSONBodyWriter` is a library meant to be used in conjunction with the
`Response` struct. This library adds convenience methods for writing JSON-encoded
response bodies.

The implementation is based on Foundry JSON manipulation cheatcodes and
therefore should be relatively performant.

This API is slightly awkward due to some Solidity and Foundry design choices.
The main inconvenience is encoding custom objects/structs into JSON which
requires a slightly more involved API. To learn how to serialize objects, check
out the [Foundry documentation](https://book.getfoundry.sh/cheatcodes/serialize-json) about `vm.serializeJson()`.

## Example
Let's assume we want to write the following JSON to the response body:
```json
{
    "user": {
        "address": "0x61880628e88b391C0161225887D65087EF5bD19B",
        "ens": "dog.eth"
    },
    "tokens": [
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
    ]
}
```

We would do it like so:
```solidity
import 'forgery-sdk/JSONBodyWriter.sol';

contract Index is Server {
    using JSONBodyWriter for Response;

    // ...

    function myRoute (
        Request calldata request
    ) public {
        // ...

        address[] memory tokens = new address[](2);
        tokens[0] = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;
        tokens[1] = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
        response.write('tokens', tokens);

        string memory userObject = vm.serializeAddress('user object', 'address', 0x61880628e88b391C0161225887D65087EF5bD19B);
        userObject = vm.serializeString('user object', 'ens', 'dog.eth');
        response.write('user', userObject);
    }
}
```

