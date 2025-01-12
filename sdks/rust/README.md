# hyveOS SDK

A Rust SDK for the [hyveOS](https://docs.p2p.industries/concepts/what-is-hyveos/) system

## Installation
Make sure you've [installed hyveOS](https://docs.p2p.industries/install/install-hyved/). Select a version of the SDK compatible with your local hyveOS version
```toml
[dependencies]
hyveos-sdk = "0.1.0"
```

## Usage
See the [service docs](https://docs.p2p.industries/sdk/request_response/) for an extensive description of the available services or the [tutorials](https://docs.p2p.industries/tutorials/hello_world_tutorial/) page for a hands-on quickstart.

## Example Usage
```rust
use hyveos_sdk::Connection;
use futures::TryStreamExt as _;

#[tokio::main]
async fn main() {
    let connection = Connection::new().await.unwrap();

    let mut req_resp_service = connection.req_resp();
    let mut requests = req_resp_service.recv(None).await.unwrap();

    while let Some((request, handle)) = requests.try_next().await.unwrap() {
        let string = String::from_utf8(request.data).unwrap();
        println!("Received request from {}: {string}", request.peer_id);
        
        handle.respond("Hello from the other side!").await.unwrap();
    }
}
```