use crate::rcm::{InvokeProps, Menu};
#[cfg(feature = "llrt")]
use llrt_modules::{fs::FsModule, os::OsModule, path::PathModule, url::UrlModule};
use rquickjs::function::This;
use rquickjs::{
    Context, Function, Module, Result, Runtime,
    function::Opt,
    loader::{BuiltinLoader, BuiltinResolver, ModuleLoader},
    module::{Declarations, Exports, ModuleDef},
};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

const LIB_MODULE: &str = include_str!("../../rcm/dist/index.js");
const DEFAULT_MODULE: &str = include_str!("../../rcm/dist/default.js");
const LITE_MODULE: &str = include_str!("../../rcm/dist/lite.js");

/// Write embedded default menu JS files to disk next to the exe.
pub fn write_menu_defaults() {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    for (name, src) in [("rcm.lite.js", LITE_MODULE), ("rcm.full.js", DEFAULT_MODULE)] {
        let path = exe_dir.join(name);
        if let Err(e) = std::fs::write(&path, src) {
            eprintln!("write_menu_defaults: write {} failed: {e}", path.display());
        } else {
            println!("write_menu_defaults: wrote {}", path.display());
        }
    }
}
///
/// `name` is `"rcm.lite"` or `"rcm.full"`.  The corresponding `.js`
/// file is looked up next to the executable.  If it exists it is used
/// as-is (allowing user customisation); otherwise the embedded default
/// is written to disk and returned.
fn load_menu_module(name: &str) -> String {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    let file_path = exe_dir.join(format!("{name}.js"));

    // Already on disk — use it
    if file_path.exists() {
        match std::fs::read_to_string(&file_path) {
            Ok(src) => {
                println!("load_menu_module: using {}.js from disk", name);
                return src;
            }
            Err(e) => eprintln!("load_menu_module: read {} failed: {e}", file_path.display()),
        }
    }

    // Not on disk — write the embedded default
    let embedded = match name {
        "rcm.lite" => LITE_MODULE,
        "rcm.full" => DEFAULT_MODULE,
        _ => DEFAULT_MODULE,
    };

    if let Err(e) = std::fs::write(&file_path, embedded) {
        eprintln!("load_menu_module: write {} failed: {e}", file_path.display());
    } else {
        println!("load_menu_module: wrote default to {}.js", name);
    }

    embedded.to_string()
}

fn print(s: String) {
    println!("{s}")
}

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
        if let Ok(Some(w)) = opts.get::<_, Option<String>>("window")
            && w.eq_ignore_ascii_case("hidden") {
                cmd.creation_flags(CREATE_NO_WINDOW);
            }
    }

    let _ = cmd.spawn(); // execute asynchronously detached
}

fn rquickjs_which(exe: String) -> Option<String> {
    if let Ok(output) = Command::new("where").arg(&exe).output()
        && output.status.success()
            && let Ok(s) = String::from_utf8(output.stdout)
                && let Some(first_line) = s.lines().next() {
                    return Some(first_line.trim().to_string());
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

pub fn invoke(props: &InvokeProps) -> std::result::Result<Menu, Box<dyn std::error::Error>> {
    let rt = Runtime::new()?;

    #[cfg(feature = "llrt")]
    let resolver = (BuiltinResolver::default()
        .with_module("rcm")
        .with_module("rcm-sys")
        .with_module("fs")
        .with_module("path")
        .with_module("url")
        .with_module("os"),);

    #[cfg(not(feature = "llrt"))]
    let resolver = (BuiltinResolver::default()
        .with_module("rcm")
        .with_module("rcm-sys"),);

    #[cfg(feature = "llrt")]
    let loader = (
        BuiltinLoader::default().with_module("rcm", LIB_MODULE),
        ModuleLoader::default()
            .with_module("fs", FsModule)
            .with_module("path", PathModule)
            .with_module("url", UrlModule)
            .with_module("os", OsModule),
    );

    #[cfg(not(feature = "llrt"))]
    let loader = (
        BuiltinLoader::default().with_module("rcm", LIB_MODULE),
        ModuleLoader::default(),
    );

    rt.set_loader(resolver, loader);

    let ctx = Context::full(&rt)?;

    ctx.with(
        |ctx| -> std::result::Result<Menu, Box<dyn std::error::Error>> {
            let global = ctx.globals();

            global
                .set("print", Function::new(ctx.clone(), print))
                .unwrap();

            // Declare our native OS binding virtual module explicitly into context natively beforehand
            Module::declare_def::<RcmSysModule, _>(ctx.clone(), "rcm-sys")?;

            // Declare the rcm index.js module
            let module = Module::declare(ctx.clone(), "rcm", LIB_MODULE)?;
            let (_, promise) = module.eval()?;
            promise.finish::<()>()?;

            // Declare the menu module (lite or full, from disk or embedded)
            let menu_name = if crate::config::is_lite() { "rcm.lite" } else { "rcm.full" };
            let menu_src = load_menu_module(menu_name);
            let module = Module::declare(ctx.clone(), "menu", menu_src.as_str())?;
            let (eval_module, promise) = module.eval()?;
            promise.finish::<()>()?;

            // Extract the default exported object (the Menu provider instance)
            let default_export: rquickjs::Value = eval_module.get("default")?;

            let props_str = serde_json::to_string(props).map_err(|e| e.to_string())?;

            // Fetch global JSON object and serialize/deserialize tools
            let json_obj: rquickjs::Object = ctx.globals().get("JSON")?;
            let parse: rquickjs::Function = json_obj.get("parse")?;
            let stringify: rquickjs::Function = json_obj.get("stringify")?;

            // Convert Rust JSON string into native QuickJS properties object
            let js_props: rquickjs::Value = parse.call((props_str,))?;

            let default_obj: rquickjs::Object = default_export
                .clone()
                .into_object()
                .ok_or("Default export is not an object")?;
            let invoke_fn: rquickjs::Function = default_obj.get("invoke")?;

            // Native explicit invocation
            let invoke_result: rquickjs::Value =
                invoke_fn.call((This(default_export.clone()), js_props))?;

            // Stringify evaluating boundaries reliably back into Rust structured Menu
            let json_str: String = stringify.call((invoke_result,))?;

            let menu_data: Menu = serde_json::from_str(&json_str)?;

            Ok(menu_data)
        },
    )
}
