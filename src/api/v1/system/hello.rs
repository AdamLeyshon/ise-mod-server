use crate::request_helpers::{ProtoBuf, ProtoBufResponseBuilder};

use actix_web::{HttpResponse, Result};
use semver_parser::version::parse;
#[cfg(test)]
use semver_parser::version::Version;

use crate::packets::hello::{HelloReply, HelloRequest};
use crate::structs::general::{CLIENT_MAX_VERSION, CLIENT_MIN_VERSION, SERVER_VERSION};

#[cfg(test)]
use actix_web::http::StatusCode;

pub async fn action_protobuf(proto_msg: ProtoBuf<HelloRequest>) -> Result<HttpResponse> {
    let client_version: String = proto_msg.0.client_version;
    match check_client_version(client_version) {
        Ok(reply) => HttpResponse::Ok().protobuf(reply),
        Err(error) => Ok(error),
    }
}

fn check_client_version(client_version: String) -> Result<HelloReply, HttpResponse> {
    let client_version = match parse(&*client_version) {
        Ok(ver) => ver,
        Err(_) => {
            return Err(HttpResponse::BadRequest().finish());
        }
    };
    if CLIENT_MIN_VERSION > client_version || client_version > CLIENT_MAX_VERSION {
        return Err(HttpResponse::UnprocessableEntity().finish());
    }
    Ok(HelloReply {
        server_version: SERVER_VERSION.to_string(),
    })
}

#[actix_web::test]
async fn test_hello_request_ok_versions() {
    use prost::Message;

    let mut min_minor_plus_1 = Version::from(CLIENT_MIN_VERSION);
    min_minor_plus_1.minor += 1;

    let tests = vec![
        HelloRequest {
            client_version: CLIENT_MIN_VERSION.to_string(),
        },
        HelloRequest {
            client_version: CLIENT_MAX_VERSION.to_string(),
        },
        HelloRequest {
            client_version: min_minor_plus_1.to_string(),
        },
    ];

    for hello_request in tests {
        // Encode Protobuf
        let mut buf = Vec::new();
        hello_request
            .encode(&mut buf)
            .expect("Failed to encode HelloRequest");

        // Send the request
        let (resp, protobuf_bytes) = test_call_hello(hello_request).await;
        assert_eq!(resp, StatusCode::OK);
        if let Some(protobuf_bytes) = protobuf_bytes {
            let hello_response: HelloReply = HelloReply::decode(protobuf_bytes.as_slice()).unwrap();
            assert_eq!(hello_response.server_version, SERVER_VERSION.to_string())
        } else {
            assert!(true, "Didn't get a response body");
        }
    }
}

#[actix_web::test]
async fn test_hello_request_bad_versions() {
    let mut min_major_minus_1 = Version::from(CLIENT_MIN_VERSION);
    min_major_minus_1.major -= 1;

    let mut max_major_plus_1 = Version::from(CLIENT_MAX_VERSION);
    max_major_plus_1.major += 1;

    let tests = vec![
        HelloRequest {
            client_version: min_major_minus_1.to_string(),
        },
        HelloRequest {
            client_version: max_major_plus_1.to_string(),
        },
    ];

    for hello_request in tests {
        let (resp, protobuf_bytes) = test_call_hello(hello_request).await;
        assert_eq!(resp, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(protobuf_bytes.is_none(),);
    }
}

#[cfg(test)]
async fn test_call_hello(message: HelloRequest) -> (StatusCode, Option<Vec<u8>>) {
    use crate::request_helpers::response_body_to_bytes;

    use prost::Message;

    // Encode Protobuf
    let mut buf = Vec::new();
    message
        .encode(&mut buf)
        .expect("Failed to encode HelloRequest");

    // Send the request
    let resp = action_protobuf(ProtoBuf(message)).await.unwrap();
    let code = resp.status();
    (
        code,
        if code == StatusCode::OK {
            Some(response_body_to_bytes(resp).await)
        } else {
            None
        },
    )
}
