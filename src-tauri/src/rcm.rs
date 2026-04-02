use rquickjs::{
    Context, Module, Runtime,
    loader::{BuiltinLoader, BuiltinResolver},
};

const LIB_MODULE: &str = include_str!("../../rcm/dist/index.js");
const DEFAULT_MODULE: &str = include_str!("../../rcm/dist/default.js");

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    pub key: Option<String>,
    pub icon: Option<String>,
    pub label: String,
    pub disable: Option<bool>,
    pub admin: Option<bool>,
    pub items: Option<Vec<Item>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconItem {
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    pub key: Option<String>,
    pub icon: Option<String>,
    pub disable: Option<bool>,
    pub admin: Option<bool>,
    pub items: Option<Vec<Item>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Menu {
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    #[serde(rename = "iconItems")]
    pub icon_items: Vec<IconItem>,
    pub groups: Vec<Group>,
}

pub fn rcm() -> std::result::Result<Menu, Box<dyn std::error::Error>> {
    // Check and setup runtime
    let rt = Runtime::new()?;

    // Resolver to map 'rcm' imports to our module name
    let resolver = BuiltinResolver::default().with_module("rcm");

    // Loader to resolve 'rcm' code string
    let loader = BuiltinLoader::default().with_module("rcm", LIB_MODULE);

    rt.set_loader(resolver, loader);

    // Create standard full context (includes standard objects like JSON)
    let ctx = Context::full(&rt)?;

    ctx.with(
        |ctx| -> std::result::Result<Menu, Box<dyn std::error::Error>> {
            // Declare the default.js module
            let module = Module::declare(ctx.clone(), "default_module", DEFAULT_MODULE)?;

            // Evaluate the module and finalize execution
            let (eval_module, promise) = module.eval()?;
            promise.finish::<()>()?;

            // Extract the default exported object (the Menu instance)
            let default_export: rquickjs::Value = eval_module.get("default")?;

            // Fetch global JSON object and stringify method
            let json_obj: rquickjs::Object = ctx.globals().get("JSON")?;
            let stringify: rquickjs::Function = json_obj.get("stringify")?;

            // Execute JSON.stringify(menu)
            let json_str: String = stringify.call((default_export,))?;

            let menu_data: Menu = serde_json::from_str(&json_str)?;

            Ok(menu_data)
        },
    )
}
