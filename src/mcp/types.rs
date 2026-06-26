use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ToolListResponse {
    pub tools: Vec<Tool>,
}

#[derive(Debug, Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    #[serde(default)]
    pub arguments: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ToolCallResponse {
    pub content: Vec<ToolContent>,
}

#[derive(Debug, Serialize)]
pub struct ToolContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}
