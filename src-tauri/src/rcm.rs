use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    #[serde(rename = "isDir")]
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvokeProps {
    pub files: Vec<FileInfo>,
    pub cwd: String,
    pub env: HashMap<String, String>,
    pub admin: bool,
    #[serde(rename = "type")]
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPayload {
    pub exe: String,
    pub args: Option<Vec<String>>,
    pub cwd: Option<String>,
    pub admin: Option<bool>,
    pub window: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub key: Option<String>,
    pub icon: Option<String>,
    pub label: Option<String>,
    pub disable: Option<bool>,
    pub admin: Option<bool>,
    pub window: Option<String>,
    pub command: Option<CommandPayload>,
    pub items: Option<Vec<Item>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Menu {
    #[serde(rename = "iconItems")]
    pub icon_items: Vec<Item>,
    pub groups: Vec<Item>,
}

pub fn invoke(props: InvokeProps) -> std::result::Result<Menu, Box<dyn std::error::Error>> {
    crate::vm::invoke(&props)
}

pub fn rcm() -> std::result::Result<Menu, Box<dyn std::error::Error>> {
    let mut env = HashMap::new();
    env.insert("OS".to_string(), "Windows".to_string());
    let props = InvokeProps {
        files: vec![],
        cwd: "C:\\".to_string(),
        env,
        admin: false,
        type_name: "Desktop".to_string(),
    };

    invoke(props)
}
