pub mod diagnostics;
pub mod ingress;
pub mod metrics;

use serde_json::Value;
use crate::mcp::types::{Tool, ToolCallResponse, ToolContent};

pub fn get_mock_ping_tool() -> Tool {
    use serde_json::json;
    Tool {
        name: "ping".to_string(),
        description: "A simple ping tool to verify connectivity.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
    }
}

pub async fn call_ping_tool(_args: Option<Value>) -> ToolCallResponse {
    ToolCallResponse {
        content: vec![ToolContent {
            content_type: "text".to_string(),
            text: "pong".to_string(),
        }],
    }
}

pub fn get_analyze_pod_failure_tool() -> Tool {
    use serde_json::json;
    Tool {
        name: "analyze_pod_failure".to_string(),
        description: "Analyzes pod failures in a specific namespace.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "namespace": { "type": "string" }
            },
            "required": ["namespace"]
        }),
    }
}

pub async fn call_analyze_pod_failure(args: Option<Value>, cluster: &dyn crate::cluster::traits::ClusterDiagnostics) -> ToolCallResponse {
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

/// Helper to safely serialize tool results, preventing panics on malformed data.
fn safe_serialize_json(data: &serde_json::Value) -> String {
    serde_json::to_string_pretty(data).unwrap_or_else(|_| "Failed to serialize diagnostic data".to_string())
}

pub fn get_check_ingress_routing_tool() -> Tool {
    ingress::get_check_ingress_routing_tool()
}

pub fn get_get_system_metrics_tool() -> Tool {
    metrics::get_get_system_metrics_tool()
}
