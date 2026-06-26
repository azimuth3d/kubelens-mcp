use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait ClusterDiagnostics: Send + Sync {
    async fn get_pod_failures(&self, namespace: &str) -> Result<Value, String>;
    async fn get_ingress_status(&self, namespace: &str) -> Result<Value, String>;
    async fn query_metrics(&self, promql: &str) -> Result<Value, String>;
}
