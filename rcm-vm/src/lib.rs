//! RCM VM — JavaScript runtime execution engine.
//! Evaluates RCM menu definitions using QuickJS (via rquickjs).
//! This crate is framework-agnostic and can be used with any frontend.

#[cfg(feature = "llrt")]
use llrt_modules::{fs::FsModule, os::OsModule, path::PathModule, url::UrlModule};
use rcm_core::{InvokeProps, Menu};
use rcm_core::{clipboard, lang};
use rquickjs::function::This;
use rquickjs::{
    Context, Function, Module, Runtime,
    loader::{BuiltinLoader, BuiltinResolver, ModuleLoader},
};

const LIB_MODULE: &str = include_str!("../../rcm-kit/dist/index.js");
const LIB_NAME: &str = "rcm-kit";
const MENU_NAME: &str = "rcm-menu";
fn print(s: String) {
    println!("{s}")
}

pub fn invoke(props: &InvokeProps) -> std::result::Result<Menu, Box<dyn std::error::Error>> {
    println!("props: {:?}", props);

    let rt = Runtime::new()?;

    let mut resolver = BuiltinResolver::default().with_module(LIB_NAME);
    let mut loader = (
        BuiltinLoader::default().with_module(LIB_NAME, LIB_MODULE),
        ModuleLoader::default(),
    );

    #[cfg(feature = "llrt")]
    {
        resolver = resolver
            .with_module("fs")
            .with_module("path")
            .with_module("url")
            .with_module("os");
        loader.1 = loader
            .1
            .with_module("fs", FsModule)
            .with_module("path", PathModule)
            .with_module("url", UrlModule)
            .with_module("os", OsModule);
    }

    rt.set_loader(resolver, loader);

    let ctx = Context::full(&rt)?;

    ctx.with(
        |ctx| -> std::result::Result<Menu, Box<dyn std::error::Error>> {
            let global = ctx.globals();

            global
                .set("print", Function::new(ctx.clone(), print))
                .unwrap();

            // Declare the rcm index.js module
            let module = Module::declare(ctx.clone(), LIB_NAME, LIB_MODULE)?;
            let (_, promise) = module.eval()?;
            promise.finish::<()>()?;

            // Declare the menu module (from disk or embedded default)
            let menu_src = rcm_core::menu::load_menu_module();
            let module = Module::declare(ctx.clone(), MENU_NAME, menu_src.as_str())?;
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

/// Build a `Menu` from a raw `ContextMenuInfo` event received from the shell.
pub fn from_info(
    info: &rcm_com::ContextMenuInfo,
) -> std::result::Result<Menu, Box<dyn std::error::Error>> {
    let mut env = std::collections::HashMap::new();
    env.insert("OS".to_string(), "Windows".to_string());

    let files: Vec<rcm_core::FileInfo> = info
        .files
        .iter()
        .map(|path| {
            let p = std::path::Path::new(path);
            rcm_core::FileInfo {
                path: path.clone(),
                is_dir: p.is_dir(),
            }
        })
        .collect();

    // Gather Start Menu / Quick Access / Autorun state (once per right-click).
    let start_menu = rcm_core::cmds::pin_to_start::list_pinned_to_start();
    let quick_access = rcm_core::cmds::quick_access::list_quick_access();
    let autorun = rcm_core::cmds::autorun::list_autorun_entries();

    let props = InvokeProps {
        files,
        cwd: info.dir.clone(),
        env,
        admin: is_admin::is_admin(),
        lang: lang::system_lang(),
        clipboard: clipboard::detect(),
        start_menu,
        quick_access,
        autorun,
    };

    invoke(&props)
}
