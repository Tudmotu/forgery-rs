# `JSONBodyParser`
The `JSONBodyParser` is a library meant to be used in conjunction with the
`Request` struct. This library adds convenience methods for reading JSON-encoded
request bodies.

The implementation is based on Foundry JSON manipulation cheatcodes and
therefore should be relatively performant.

The API should feel similar to other typed-languages JSON handling utils.

## Example
Let's assume the request contains the following JSON body:
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

We could read the values like so:
```solidity
import 'forgery-sdk/JSONBodyParser.sol';

contract Index is Server {
    using JSONBodyParser for Request;

    // ...

    function myRoute (
        Request calldata request
    ) public {
        address userAddress = request.json().at('user').at('address').asAddress();
        string memory ens = request.json().at('user').at('ens').asString();
        address[] memory tokens = request.json().at('tokens').asAddressArray();

        // ...
    }
}
```
