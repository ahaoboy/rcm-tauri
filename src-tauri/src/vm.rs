use crate::rcm::{InvokeProps, Menu};
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
    let resolver = (BuiltinResolver::default()
        .with_module("rcm")
        .with_module("rcm-sys")
        .with_module("fs")
        .with_module("path")
        .with_module("url")
        .with_module("os"),);

    let loader = (
        BuiltinLoader::default().with_module("rcm", LIB_MODULE),
        ModuleLoader::default()
            .with_module("fs", FsModule)
            .with_module("path", PathModule)
            .with_module("url", UrlModule)
            .with_module("os", OsModule),
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

            // Declare the default.js menu module
            let module = Module::declare(ctx.clone(), "menu", DEFAULT_MODULE)?;
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
