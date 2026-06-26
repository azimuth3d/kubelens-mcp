# kube-rs v1.0 Syntax Reference

## Client Initialization
```rust
use kube::Client;
let client = Client::try_default().await?;
```

## API Access
```rust
use kube::{api::{Api, ListParams}, Client};
use k8s_openapi::api::v1::Pod;

let pods: Api<Pod> = Api::namespaced(client, "default");
let list = pods.list(&ListParams::default()).await?;
```

## Error Handling
Map `kube::Error` to `String` for MCP responses.
```rust
match result {
    Ok(val) => Ok(serde_json::json!(val)),
    Err(e) => Err(format!("Kube error: {}", e)),
}
```
