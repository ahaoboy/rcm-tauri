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
    /// Current i18n language (e.g. 'en', 'zh'). Falls back to 'en' if unsupported.
    pub lang: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPayload {
    pub exe: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub cwd: String,
    #[serde(default)]
    pub admin: bool,
    #[serde(default)]
    pub window: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub disable: bool,
    #[serde(default)]
    pub admin: bool,
    #[serde(default)]
    pub window: String,
    #[serde(default)]
    pub items: Vec<Item>,
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
        lang: crate::lang::system_lang(),
    };

    invoke(props)
}
