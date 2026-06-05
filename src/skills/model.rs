use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub has_agents_dir: bool,
    pub file_count: usize,
}
