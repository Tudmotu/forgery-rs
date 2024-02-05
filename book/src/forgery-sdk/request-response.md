# Request & Response
The `Request` and `Response` objects are simple structs with added user-defined
Solidity methods.

These structs are very simple and have the following definitions:
```solidity
struct Header {
    string key;
    string value;
}

struct Request {
    string method;
    string uri;
    Header[] headers;
    bytes body;
}

struct Response {
    uint16 status;
    Header[] headers;
    bytes body;
}
```

In addition, they each have a method to help read/write headers.

## `Request`
```solidity
contract Index is Server {
    // ...

    function hello (
        Request calldata request
    ) public {
        string memory contentType = request.header('content-type');
    }
}
```

## `Response`
```solidity
contract Index is Server {
    // ...

    function hello (
        Request calldata request
    ) public {
        response.header('content-type', 'application/json');
    }
}
```
