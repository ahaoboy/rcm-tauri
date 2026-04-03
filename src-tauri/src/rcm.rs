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
    #[serde(default, deserialize_with = "deserialize_command")]
    pub command: Option<Vec<CommandPayload>>,
    pub items: Option<Vec<Item>>,
}

fn deserialize_command<'de, D>(deserializer: D) -> std::result::Result<Option<Vec<CommandPayload>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum SingleOrVec {
        Single(CommandPayload),
        Vec(Vec<CommandPayload>),
    }

    match Option::<SingleOrVec>::deserialize(deserializer)? {
        Some(SingleOrVec::Single(c)) => Ok(Some(vec![c])),
        Some(SingleOrVec::Vec(v)) => Ok(Some(v)),
        None => Ok(None),
    }
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
