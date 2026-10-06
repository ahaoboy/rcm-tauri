//! RCM VM — JavaScript runtime execution engine.
//! Evaluates RCM menu definitions using QuickJS (via rquickjs).
//! This crate is framework-agnostic and can be used with any frontend.

#[cfg(feature = "llrt")]
use llrt_modules::{fs::FsModule, os::OsModule, path::PathModule, url::UrlModule};
use rcm_core::{InvokeProps, Menu};
use rcm_core::{clipboard, lang};
use rquickjs::function::This;
use rquickjs::{
    Context, Ctx, Function, Module, Runtime,
    loader::{BuiltinLoader, BuiltinResolver, ModuleLoader},
};

/// The `rcm-kit` runtime, bundled into the binary.
///
/// This path must stay inside the crate: cargo only ships files under the crate
/// directory, so a path like `../../rcm-kit/dist/index.js` resolves when building
/// from a checkout of this repo but fails for anyone consuming the crate.
///
/// The file is generated from `rcm-kit` by `bun build:rcm`; CI verifies it has
/// not gone stale.
const LIB_MODULE: &str = include_str!("../assets/index.js");
const LIB_NAME: &str = "rcm-kit";
const MENU_NAME: &str = "rcm-menu";
fn print(s: String) {
    println!("{s}")
}

pub fn invoke(props: &InvokeProps) -> std::result::Result<Menu, Box<dyn std::error::Error>> {
    println!("props: {:?}", props);

    let rt = Runtime::new()?;

    let resolver = BuiltinResolver::default().with_module(LIB_NAME);
    let loader = (
        BuiltinLoader::default().with_module(LIB_NAME, LIB_MODULE),
        ModuleLoader::default(),
    );

    // Shadow the bindings when `llrt` is enabled. Doing it here instead of with
    // `mut` keeps both feature configurations free of `unused_mut` warnings.
    #[cfg(feature = "llrt")]
    let (resolver, loader) = (
        resolver
            .with_module("fs")
            .with_module("path")
            .with_module("url")
            .with_module("os"),
        (
            loader.0,
            loader
                .1
                .with_module("fs", FsModule)
                .with_module("path", PathModule)
                .with_module("url", UrlModule)
                .with_module("os", OsModule),
        ),
    );

    rt.set_loader(resolver, loader);

    let ctx = Context::full(&rt)?;
    let menu_src = rcm_core::menu::load_menu_module();

    // `String` boxes into `Box<dyn Error>` directly, so the public signature is unchanged.
    ctx.with(|ctx| eval_menu(&ctx, &menu_src, props))
        .map_err(Into::into)
}

/// Evaluate the menu module `menu_src` in `ctx` and run its `invoke` with `props`.
///
/// Split out of [`invoke`] so the failure paths can be tested without touching the
/// menu file on disk.
fn eval_menu<'js>(
    ctx: &Ctx<'js>,
    menu_src: &str,
    props: &InvokeProps,
) -> std::result::Result<Menu, String> {
    // A JS failure surfaces as `Error::Exception`, a bare marker whose payload
    // (message + stack) stays pending in the context and is only reachable through
    // `catch()`. Propagating the raw error would print just the word "Exception".
    let js_err = |e: rquickjs::Error| rquickjs::CaughtError::from_error(ctx, e).to_string();

    let global = ctx.globals();

    global
        .set("print", Function::new(ctx.clone(), print))
        .map_err(js_err)?;

    // Declare the rcm index.js module
    let module = Module::declare(ctx.clone(), LIB_NAME, LIB_MODULE).map_err(js_err)?;
    let (_, promise) = module.eval().map_err(js_err)?;
    promise.finish::<()>().map_err(js_err)?;

    // Declare the menu module (from disk or embedded default)
    let module = Module::declare(ctx.clone(), MENU_NAME, menu_src).map_err(js_err)?;
    let (eval_module, promise) = module.eval().map_err(js_err)?;
    promise.finish::<()>().map_err(js_err)?;

    // Extract the default exported object (the Menu provider instance)
    let default_export: rquickjs::Value = eval_module.get("default").map_err(js_err)?;

    let props_str = serde_json::to_string(props).map_err(|e| e.to_string())?;

    // Cross the Rust/JS boundary with the context's own JSON, so the module sees
    // a real object rather than a string.
    let json_obj: rquickjs::Object = ctx.globals().get("JSON").map_err(js_err)?;
    let parse: rquickjs::Function = json_obj.get("parse").map_err(js_err)?;
    let stringify: rquickjs::Function = json_obj.get("stringify").map_err(js_err)?;
    let js_props: rquickjs::Value = parse.call((props_str,)).map_err(js_err)?;

    let default_obj: rquickjs::Object = default_export
        .clone()
        .into_object()
        .ok_or_else(|| "menu module default export is not an object".to_string())?;
    let invoke_fn: rquickjs::Function = default_obj.get("invoke").map_err(js_err)?;

    let invoke_result: rquickjs::Value = invoke_fn
        .call((This(default_export.clone()), js_props))
        .map_err(js_err)?;

    let json_str: String = stringify.call((invoke_result,)).map_err(js_err)?;

    serde_json::from_str(&json_str).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Evaluate `menu_src` in a fresh runtime and return the error message, if any.
    fn error_for(menu_src: &str) -> String {
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        let props = InvokeProps {
            files: Vec::new(),
            cwd: String::new(),
            env: std::collections::HashMap::new(),
            admin: false,
            lang: "en".to_string(),
            clipboard: Default::default(),
            startmenu: Default::default(),
            quick_access: Default::default(),
            autorun: Default::default(),
            desktop: Default::default(),
        };
        ctx.with(|ctx| eval_menu(&ctx, menu_src, &props))
            .expect_err("expected the module to fail")
    }

    #[test]
    fn a_thrown_error_reports_its_message() {
        let msg = error_for("export default { invoke() { throw new Error('boom') } }");
        assert!(msg.contains("boom"), "message lost, got: {msg}");
    }

    #[test]
    fn a_syntax_error_reports_its_location() {
        let msg = error_for("export default { invoke( { } }");
        assert!(msg.contains("rcm-menu"), "location lost, got: {msg}");
    }

    #[test]
    fn a_missing_invoke_export_is_reported() {
        let msg = error_for("export default {}");
        assert!(msg.contains("invoke"), "message lost, got: {msg}");
    }
}

/// Build a `Menu` from a raw `ContextMenuInfo` event received from the shell.
pub fn from_info(
    info: &rcm_com::ContextMenuInfo,
) -> std::result::Result<Menu, Box<dyn std::error::Error>> {
    let mut env = std::collections::HashMap::new();
    env.insert("OS".to_string(), "Windows".to_string());
    // Common user folder locations (HOME, DESKTOP, DOCUMENTS, …) so menu JS
    // can reference them via props.env.
    env.extend(rcm_core::paths::common());

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

    // Gather Start Menu / Quick Access / Autorun / Desktop state (once per right-click).
    let startmenu = rcm_core::cmds::pin_to_start::list_pinned_to_start();
    let quick_access = rcm_core::cmds::quick_access::list_quick_access();
    let autorun = rcm_core::cmds::autorun::list_autorun_entries();
    let desktop = rcm_core::cmds::desktop::list();

    let props = InvokeProps {
        files,
        cwd: info.dir.clone(),
        env,
        admin: is_admin::is_admin(),
        lang: rcm_core::config::lang().unwrap_or_else(lang::system_lang),
        clipboard: clipboard::detect(),
        startmenu,
        quick_access,
        autorun,
        desktop,
    };

    invoke(&props)
}
