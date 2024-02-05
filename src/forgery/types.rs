use core::str::FromStr;

use alloy_sol_macro::sol;
use http_body_util::{BodyExt, Full};
use hyper::{
    body::{Bytes as HyperBytes, Incoming},
    header::{HeaderName, HeaderValue},
    HeaderMap, Request, Response, StatusCode,
};

sol! {
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

struct ForgeryHeaderMap {
    headers: HeaderMap,
}

impl From<&HeaderMap> for ForgeryHeaderMap {
    fn from(headers: &HeaderMap) -> Self {
        ForgeryHeaderMap {
            headers: headers.clone(),
        }
    }
}

impl From<Vec<SolHttpHeader>> for ForgeryHeaderMap {
    fn from(headers: Vec<SolHttpHeader>) -> Self {
        let mut map = HeaderMap::new();
        for header in headers.clone() {
            map.insert(
                HeaderName::from_str(&header.key).unwrap(),
                HeaderValue::from_str(&header.value).unwrap(),
            );
        }
        ForgeryHeaderMap {
            headers: map.clone(),
        }
    }
}

impl From<ForgeryHeaderMap> for Vec<SolHttpHeader> {
    fn from(val: ForgeryHeaderMap) -> Self {
        val.headers
            .iter()
            .map(|(key, value)| SolHttpHeader {
                key: key.to_string(),
                value: value.to_str().unwrap().to_string(),
            })
            .collect()
    }
}

impl SolHttpRequest {
    pub async fn from_incoming(req: Request<Incoming>) -> Result<Self, hyper::Error> {
        let method = req.method().to_string();
        let uri = req.uri().to_string();
        let headers = ForgeryHeaderMap::from(req.headers()).into();
        let bytes = req.collect().await?.to_bytes();
        let body = bytes.iter().cloned().collect::<Vec<u8>>();
        Ok(SolHttpRequest {
            method,
            uri,
            headers,
            body,
        })
    }
}

impl From<SolHttpResponse> for Response<Full<HyperBytes>> {
    fn from(val: SolHttpResponse) -> Self {
        let mut builder =
            Response::builder().status(&StatusCode::from_u16(val.status).unwrap_or_else(|err| {
                println!("Malformed response from index contract: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            }));

        builder
            .headers_mut()
            .expect("Failed to generate response headers")
            .extend(ForgeryHeaderMap::from(val.headers).headers);

        builder
            .body(Full::new(HyperBytes::from(val.body)))
            .expect("Failed to decode SolHttpResponse into hyper::Response")
    }
}
