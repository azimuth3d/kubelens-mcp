# k8s-openapi v0.23 Syntax Reference

## Pod Structure Access
```rust
use k8s_openapi::api::v1::Pod;

// Accessing metadata safely
let name = pod.metadata.name.clone().unwrap_or_default();

// Accessing container statuses
if let Some(status) = &pod.status {
    for cs in &status.container_statuses {
        // Check state.last_state or state.waiting/terminated
    }
}
```

## Feature Flags
Always enable a specific Kubernetes version feature (e.g., `v1_30`) to match your cluster API.
