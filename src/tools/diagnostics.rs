use serde_json::Value;
use crate::cluster::traits::ClusterDiagnostics;
use crate::mcp::types::{ToolCallResponse, ToolContent};

pub async fn analyze_pod_failure(cluster: &dyn ClusterDiagnostics, args: Option<Value>) -> ToolCallResponse {
    let args = match args {
        Some(a) => a,
        None => return ToolCallResponse { content: vec![ToolContent { content_type: "text".to_string(), text: "Missing arguments.".to_string() }] }
    };
    
    let namespace = match args.get("namespace").and_then(|v| v.as_str()) {
        Some(ns) => ns,
        None => return ToolCallResponse { content: vec![ToolContent { content_type: "text".to_string(), text: "Missing 'namespace' argument.".to_string() }] }
    };

    match cluster.get_pod_failures(namespace).await {
        Ok(data) => ToolCallResponse {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: serde_json::to_string_pretty(&data).unwrap_or_else(|_| "Failed to serialize data".to_string()),
            }],
        },
        Err(e) => ToolCallResponse {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: format!("Error analyzing pod failures: {}", e),
            }],
        }
    }
}
