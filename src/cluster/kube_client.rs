use async_trait::async_trait;
use kube::{api::{Api, ListParams}, Client};
use k8s_openapi::api::core::v1::Pod;
use serde_json::Value;

use crate::cluster::traits::ClusterDiagnostics;

/// Adapter for live Kubernetes cluster diagnostics.
/// All infrastructure errors are caught and mapped to `Result<Value, String>` 
/// to prevent panics. The MCP layer maps these strings to JSON-RPC `-32603` frames.
pub struct KubeSdkAdapter {
    client: Client,
    http_client: reqwest::Client,
}

impl KubeSdkAdapter {
    pub async fn new() -> Result<Self, String> {
        let client = Client::try_default().await.map_err(|e| format!("Failed to init kube client: {}", e))?;
        Ok(Self {
            client,
            http_client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl ClusterDiagnostics for KubeSdkAdapter {
    async fn get_pod_failures(&self, namespace: &str) -> Result<Value, String> {
        let pods_api: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        let list_params = ListParams::default();
        
        let pod_list = pods_api.list(&list_params).await.map_err(|e| format!("Failed to list pods: {}", e))?;
        
        let mut failures = Vec::new();
        for pod in &pod_list.items {
            if let Some(status) = &pod.status {
                if let Some(container_statuses) = &status.container_statuses {
                    for cs in container_statuses.iter() {
                        if let Some(last_state) = &cs.last_state {
                            if last_state.terminated.is_some() || last_state.waiting.is_some() {
                                failures.push(serde_json::json!({
                                    "pod": pod.metadata.name.clone().unwrap_or_default(),
                                    "container": cs.name.clone(),
                                    "reason": "Previous state indicates failure"
                                }));
                            }
                        }
                        if let Some(waiting) = &cs.state {
                            if waiting.waiting.is_some() {
                                failures.push(serde_json::json!({
                                    "pod": pod.metadata.name.clone().unwrap_or_default(),
                                    "container": cs.name.clone(),
                                    "reason": "Currently waiting"
                                }));
                            }
                        }
                    }
                }
            }
        }

        Ok(serde_json::json!({
            "namespace": namespace,
            "failures": failures,
            "total_pods_checked": pod_list.items.len()
        }))
    }

    async fn get_ingress_status(&self, namespace: &str) -> Result<Value, String> {
        let apisix_path = format!("/apis/apisix.apache.org/v1beta2/namespaces/{}/apisixroutes", namespace);
        let cert_path = format!("/apis/cert-manager.io/v1/namespaces/{}/certificates", namespace);

        let base_url = std::env::var("KUBE_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
        let client = &self.http_client;

        let apisix_routes: Value = client.get(&format!("{}{}", base_url, apisix_path))
            .send().await.map_err(|e| format!("Failed to fetch APISIX routes: {}", e))?
            .json().await.map_err(|e| format!("Failed to parse APISIX response: {}", e))?;
            
        let certificates: Value = client.get(&format!("{}{}", base_url, cert_path))
            .send().await.map_err(|e| format!("Failed to fetch certificates: {}", e))?
            .json().await.map_err(|e| format!("Failed to parse certificates response: {}", e))?;

        Ok(serde_json::json!({
            "apisix_routes": apisix_routes,
            "certificates": certificates
        }))
    }

    async fn query_metrics(&self, promql: &str) -> Result<Value, String> {
        let prometheus_url = std::env::var("PROMETHEUS_URL").unwrap_or_else(|_| "http://localhost:9090".to_string());
        let url = format!("{}/api/v1/query", prometheus_url);

        let response = self.http_client.get(&url)
            .query(&[("query", promql)])
            .send()
            .await
            .map_err(|e| format!("Failed to query Prometheus: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Prometheus API error: {}", response.status()));
        }

        let json: Value = response.json().await.map_err(|e| format!("Failed to parse Prometheus response: {}", e))?;
        Ok(json)
    }
}
