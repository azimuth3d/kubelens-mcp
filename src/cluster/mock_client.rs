use super::traits::ClusterDiagnostics;
use async_trait::async_trait;
use serde_json::Value;

pub struct MockClusterClient;

#[async_trait]
impl ClusterDiagnostics for MockClusterClient {
    async fn get_pod_failures(&self, _namespace: &str) -> Result<Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "message": "No failures detected in mock environment.",
            "pods": []
        }))
    }

    async fn get_ingress_status(&self, _namespace: &str) -> Result<Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "message": "Ingress routing is healthy in mock environment.",
            "routes": []
        }))
    }

    async fn query_metrics(&self, _promql: &str) -> Result<Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "message": "Metrics query successful in mock environment.",
            "data": []
        }))
    }
}
