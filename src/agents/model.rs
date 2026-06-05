use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Agent {
    pub name: String,
    pub description: String,
    pub model: Option<String>,
    pub tools: Vec<String>,
}
