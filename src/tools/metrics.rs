// Placeholder for Phase 4: Prometheus queries execution
use crate::cluster::traits::ClusterDiagnostics;
use crate::mcp::types::Tool;
use serde_json::{json, Value};

pub async fn get_system_metrics(cluster: &dyn ClusterDiagnostics, args: Value) -> Result<Value, String> {
    let promql = args.get("promql")
        .and_then(|v| v.as_str())
        .ok_or("Missing or invalid 'promql' parameter".to_string())?;

    let metrics = cluster.query_metrics(promql).await?;
    Ok(json!({
        "status": "success",
        "data": metrics,
        "message": "System metrics retrieved successfully."
    }))
}

pub fn get_get_system_metrics_tool() -> Tool {
    Tool {
        name: "get_system_metrics".to_string(),
        description: "Queries Prometheus for raw system or application metrics using PromQL.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "promql": {
                    "type": "string",
                    "description": "Prometheus Query Language (PromQL) expression to execute."
                }
            },
            "required": ["promql"]
        })
    }
}
