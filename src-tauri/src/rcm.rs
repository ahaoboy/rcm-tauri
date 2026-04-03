use rquickjs::{
    Context, Module, Result, Runtime,
    function::Opt,
    loader::{BuiltinLoader, BuiltinResolver},
    module::{Declarations, Exports, ModuleDef},
};
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

fn rquickjs_run<'js>(exe: String, args: Opt<Vec<String>>, options: Opt<rquickjs::Object<'js>>) {
    let mut cmd = Command::new(exe);

    if let Some(a) = args.0 {
        cmd.args(a);
    }

    if let Some(opts) = options.0 {
        if let Ok(Some(c)) = opts.get::<_, Option<String>>("cwd") {
            cmd.current_dir(c);
        }

        #[cfg(target_os = "windows")]
        if let Ok(Some(w)) = opts.get::<_, Option<String>>("window") {
            if w.eq_ignore_ascii_case("hidden") {
                cmd.creation_flags(CREATE_NO_WINDOW);
            }
        }
    }

    let _ = cmd.spawn(); // execute asynchronously detached
}

fn rquickjs_which(exe: String) -> Option<String> {
    if let Ok(output) = Command::new("where").arg(&exe).output() {
        if output.status.success() {
            if let Ok(s) = String::from_utf8(output.stdout) {
                if let Some(first_line) = s.lines().next() {
                    return Some(first_line.trim().to_string());
                }
            }
        }
    }
    None
}

fn rquickjs_find_unique_path(dir: String, name: String) -> String {
    let base_path = Path::new(&dir).join(&name);
    if !base_path.exists() {
        return base_path.to_string_lossy().to_string();
    }

    let extension = base_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let stem = base_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&name);

    let mut counter = 2;
    loop {
        let new_name = if extension.is_empty() {
            format!("{}({})", stem, counter)
        } else {
            format!("{}({}).{}", stem, counter, extension)
        };

        let new_path = Path::new(&dir).join(&new_name);
        if !new_path.exists() {
            return new_path.to_string_lossy().to_string();
        }
        counter += 1;
    }
}

pub struct RcmSysModule;

impl ModuleDef for RcmSysModule {
    fn declare(declare: &Declarations) -> Result<()> {
        declare.declare("run")?;
        declare.declare("which")?;
        declare.declare("where")?;
        declare.declare("findUniquePath")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &rquickjs::Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export("run", rquickjs::Function::new(ctx.clone(), rquickjs_run)?)?;
        exports.export(
            "which",
            rquickjs::Function::new(ctx.clone(), rquickjs_which)?,
        )?;
        exports.export(
            "where",
            rquickjs::Function::new(ctx.clone(), rquickjs_which)?,
        )?;
        exports.export(
            "findUniquePath",
            rquickjs::Function::new(ctx.clone(), rquickjs_find_unique_path)?,
        )?;
        Ok(())
    }
}

const LIB_MODULE: &str = include_str!("../../rcm/dist/index.js");
const DEFAULT_MODULE: &str = include_str!("../../rcm/dist/default.js");

use serde::{Deserialize, Serialize};

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
            // Declare our native OS binding virtual module explicitly into context natively beforehand
            Module::declare_def::<RcmSysModule, _>(ctx.clone(), "rcm-sys")?;

            // Declare the default.js module
            let module = Module::declare(ctx.clone(), "default_module", DEFAULT_MODULE)?;

            // Evaluate the module and finalize execution
            let (eval_module, promise) = module.eval()?;
            promise.finish::<()>()?;

            // Extract the default exported object (the Menu instance)
            let default_export: rquickjs::Value = eval_module.get("default")?;

            // Generate contextual InvokeProps attributes reflecting system bindings
            let mut env = HashMap::new();
            env.insert("OS".to_string(), "Windows".to_string());
            let props = InvokeProps {
                files: vec![],
                cwd: "C:\\".to_string(),
                env,
                admin: false,
                type_name: "Desktop".to_string(),
            };

            let props_str = serde_json::to_string(&props).map_err(|e| e.to_string())?;

            // Fetch global JSON object and stringify method
            let json_obj: rquickjs::Object = ctx.globals().get("JSON")?;
            let parse: rquickjs::Function = json_obj.get("parse")?;
            let stringify: rquickjs::Function = json_obj.get("stringify")?;

            // Safely parse properties string converting to QuickJS object values
            let js_props: rquickjs::Value = parse.call((props_str,))?;

            // Mount values implicitly to Global avoiding Function instance this binding loss bounds
            ctx.globals().set("__menu_target__", default_export)?;
            ctx.globals().set("__menu_props__", js_props)?;

            let invoke_result: rquickjs::Value =
                ctx.eval("__menu_target__.invoke(__menu_props__)")?;

            // Execute JSON.stringify mapping explicit results gracefully
            let json_str: String = stringify.call((invoke_result,))?;

            let menu_data: Menu = serde_json::from_str(&json_str)?;

            Ok(menu_data)
        },
    )
}
