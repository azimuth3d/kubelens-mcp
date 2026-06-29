mod cluster;
mod mcp;
mod tools;

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use serde_json::Value;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let use_mock = std::env::var("KUBELENS_USE_MOCK").unwrap_or_default() == "true";
    let cluster: Arc<dyn cluster::traits::ClusterDiagnostics> = if use_mock {
        Arc::new(cluster::mock_client::MockClusterClient)
    } else {
        match cluster::kube_client::KubeSdkAdapter::new().await {
            Ok(adapter) => Arc::new(adapter),
            Err(e) => {
                eprintln!(
                    "Failed to initialize live cluster client: {}. Falling back to mock.",
                    e
                );
                Arc::new(cluster::mock_client::MockClusterClient)
            }
        }
    };

    let app = Router::new()
        .route("/mcp", post(handle_mcp_request))
        .with_state(cluster);

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("MCP Server listening on http://0.0.0.0:8080");
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_mcp_request(
    State(cluster): State<Arc<dyn cluster::traits::ClusterDiagnostics>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    match mcp::handle_request(&payload.to_string(), cluster.as_ref()).await {
        Some(response_str) => {
            let response_value: Value = serde_json::from_str(&response_str)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(response_value))
        }
        None => {
            let error_id = payload.get("id").cloned().unwrap_or(serde_json::json!(null));
            let error_response = serde_json::json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32603,
                    "message": "Internal error processing request"
                },
                "id": error_id
            });
            Ok(Json(error_response))
        }
    }
}
