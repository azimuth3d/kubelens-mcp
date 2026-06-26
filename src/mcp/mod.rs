pub mod protocol;
pub mod types;

use serde_json::{json, Value};
use crate::mcp::protocol::{JsonRpcRequest, JsonRpcResponse};
use crate::cluster::traits::ClusterDiagnostics;

pub async fn handle_request(line: &str, cluster: &dyn ClusterDiagnostics) -> Option<String> {
    let request = match serde_json::from_str::<JsonRpcRequest>(line) {
        Ok(req) => req,
        Err(e) => return Some(serialize_error(Value::Null, -32700, format!("Parse error: {}", e))),
    };

    let id = request.id.clone().unwrap_or(Value::Null);
    let method = request.method.clone();
    let params = request.params;

    let response = match method.as_str() {
        "initialize" => {
            let id_val = id.clone();
            handle_initialize(id_val)
        }
        "notifications/initialized" => {
            let id_val = id.clone();
            JsonRpcResponse::success(id_val, Value::Null)
        }
        "tools/list" => {
            let id_val = id.clone();
            handle_tools_list(id_val)
        }
        "tools/call" => {
            let id_val = id.clone();
            handle_tools_call(params, id_val, cluster).await
        }
        _ => {
            let id_val = id.clone();
            JsonRpcResponse::error(&id_val, -32601, format!("Method not found: {}", method))
        }
    };

    Some(serialize_response(&response))
}

fn serialize_error(id: Value, code: i32, message: String) -> String {
    serde_json::to_string(&JsonRpcResponse::error(&id, code, message)).unwrap_or_else(|_| 
        serde_json::to_string(&JsonRpcResponse::error(&id, -32603, "Internal serialization error".to_string())).unwrap_or_default()
    )
}

fn serialize_response(response: &JsonRpcResponse) -> String {
    serde_json::to_string(response).unwrap_or_else(|_| 
        serde_json::to_string(&JsonRpcResponse::error(&Value::Null, -32603, "Internal serialization error".to_string())).unwrap_or_default()
    )
}

fn handle_initialize(id: Value) -> JsonRpcResponse {
    let server_info = json!({
        "name": "kubelens-mcp",
        "version": "0.1.0"
    });
    let protocol_version = "2024-11-05"; 
    let capabilities = json!({});

    JsonRpcResponse::success(id, json!({
        "protocolVersion": protocol_version,
        "capabilities": capabilities,
        "serverInfo": server_info
    }))
}

fn handle_tools_list(id: Value) -> JsonRpcResponse {
    use crate::mcp::types::ToolListResponse;
    let response = ToolListResponse {
        tools: vec![
            crate::tools::get_mock_ping_tool(),
            crate::tools::get_analyze_pod_failure_tool(),
            crate::tools::get_check_ingress_routing_tool(),
            crate::tools::get_get_system_metrics_tool(),
        ],
    };
    JsonRpcResponse::success(id, serde_json::to_value(response).unwrap_or_else(|e| {
        eprintln!("Failed to serialize tool list: {}", e);
        serde_json::json!({"error": "Serialization failed"})
    }))
}

async fn handle_tools_call(params: Option<Value>, id: Value, cluster: &dyn ClusterDiagnostics) -> JsonRpcResponse {
    let id = id.clone();
    use crate::mcp::types::ToolCallParams;
    let params = match params {
        Some(p) => match serde_json::from_value::<ToolCallParams>(p) {
            Ok(p) => p,
            Err(e) => return JsonRpcResponse::error(&id, -32602, format!("Invalid params: {}", e)),
        },
        None => return JsonRpcResponse::error(&id, -32602, "Missing params".to_string()),
    };

    match params.name.as_str() {
        "ping" => {
            let res = crate::tools::call_ping_tool(params.arguments).await;
            JsonRpcResponse::success(id, serde_json::to_value(res).unwrap_or_else(|_| json!({"error": "Serialization failed"})))
        },
        "analyze_pod_failure" => {
            let res = crate::tools::call_analyze_pod_failure(params.arguments, cluster).await;
            JsonRpcResponse::success(id, serde_json::to_value(res).unwrap_or_else(|_| json!({"error": "Serialization failed"})))
        },
        "check_ingress_routing" => {
            let args = params.arguments.clone().unwrap_or(json!({}));
            match crate::tools::ingress::check_ingress_routing(cluster, args).await {
                Ok(data) => JsonRpcResponse::success(id, data),
                Err(e) => JsonRpcResponse::error(&id, -32603, e),
            }
        },
        "get_system_metrics" => {
            let args = params.arguments.clone().unwrap_or(json!({}));
            match crate::tools::metrics::get_system_metrics(cluster, args).await {
                Ok(data) => JsonRpcResponse::success(id, data),
                Err(e) => JsonRpcResponse::error(&id, -32603, e),
            }
        },
        _ => return JsonRpcResponse::error(&id, -32601, format!("Unknown tool: {}", params.name)),
    }
}
