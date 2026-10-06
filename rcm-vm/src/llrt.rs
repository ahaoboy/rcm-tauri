//! The QuickJS engine (via `rquickjs`) — selected by the `llrt` feature.
//!
//! The `llrt` feature is what brings in `llrt_modules`, which is where the Node
//! built-ins `rcm-kit` imports (`fs`/`path`/`os`) come from; there is nothing to
//! gate inside this module, since it only exists when they are available.
//!
//! The shape an engine has to have — build once, evaluate per call — is
//! [`crate::engine::Engine`]; this module is only the QuickJS-specific half.
//! `Context` owns its `Runtime`, so keeping it alive keeps the loader, the
//! built-in modules and the evaluated menu alive with it.

use llrt_modules::{fs::FsModule, os::OsModule, path::PathModule};
use rquickjs::function::This;
use rquickjs::{
    Context, Ctx, Function, Module, Runtime,
    loader::{BuiltinLoader, BuiltinResolver, ModuleLoader},
};

use crate::engine::Engine;
use crate::{LIB_MODULE, LIB_NAME, MENU_NAME};

/// Global the menu module object is parked in between invocations.
///
/// The module is declared on every call, but each declaration needs somewhere
/// for its default export to land, and the per-call script needs somewhere to
/// find it.
const MENU_GLOBAL: &str = "__rcm_menu";

/// A live QuickJS context with `rcm-kit` evaluated.
pub(crate) struct QuickJsEngine {
    /// Keeps the `Runtime` alive; dropping it drops the whole engine.
    ctx: Context,
}

impl Engine for QuickJsEngine {
    fn init(menu_source: &str) -> Result<Self, String> {
        let rt = Runtime::new().map_err(|e| e.to_string())?;

        let resolver = BuiltinResolver::default()
            .with_module(LIB_NAME)
            .with_module("fs")
            .with_module("path")
            .with_module("os");
        let loader = (
            BuiltinLoader::default().with_module(LIB_NAME, LIB_MODULE),
            ModuleLoader::default()
                .with_module("fs", FsModule)
                .with_module("path", PathModule)
                .with_module("os", OsModule),
        );
        rt.set_loader(resolver, loader);

        let ctx = Context::full(&rt).map_err(|e| e.to_string())?;
        ctx.with(|ctx| {
            globals(&ctx)?;
            bundle(&ctx)?;
            // Loaded once per call, and this is also the runtime's warm-up.
            let _ = load_menu(&ctx, menu_source);
            Ok(())
        })?;

        Ok(Self { ctx })
    }

    fn evaluate(&mut self, menu_source: &str, props_json: &str) -> Result<String, String> {
        // The menu is declared and evaluated here, from the source just read, so
        // whatever `rcm.js` holds now is what runs.
        self.ctx.with(|ctx| {
            load_menu(&ctx, menu_source)?;
            invoke(&ctx, props_json)
        })
    }
}

/// `rcm-kit` logs through a bare `print`, which is a QuickJS global.
fn print(s: String) {
    println!("{s}");
}

/// Renders QuickJS errors with their message and stack.
///
/// A JS failure surfaces as `Error::Exception`, a bare marker whose payload stays
/// pending in the context and is only reachable through `catch()`. Propagating
/// the raw error would print just the word "Exception".
fn error_renderer(ctx: &Ctx<'_>) -> impl Fn(rquickjs::Error) -> String + '_ {
    move |e| rquickjs::CaughtError::from_error(ctx, e).to_string()
}

/// Register the `print` global. Done once, before the first module is evaluated.
fn globals(ctx: &Ctx<'_>) -> Result<(), String> {
    ctx.globals()
        .set("print", Function::new(ctx.clone(), print))
        .map_err(error_renderer(ctx))
}

/// Evaluate `rcm-kit` once. It is compiled into the binary, so it never changes
/// and is never evaluated again; doing it here also warms the module up.
fn bundle(ctx: &Ctx<'_>) -> Result<(), String> {
    let js_err = error_renderer(ctx);
    let bundle = Module::declare(ctx.clone(), LIB_NAME, LIB_MODULE).map_err(js_err)?;
    let (_, promise) = bundle.eval().map_err(js_err)?;
    promise.finish::<()>().map_err(js_err)
}

/// Evaluate the menu module and park its default export in [`MENU_GLOBAL`],
/// replacing whatever the previous call left there.
///
/// Declaring is what compiles, so a declaration made here is a module built from
/// the source just read — there is no cached module to invalidate, and the name
/// it is declared under only has to stay consistent for error messages and for
/// resolving what it imports. The `rcm-kit` behind it keeps the identity it got
/// at startup and is not re-evaluated.
fn load_menu(ctx: &Ctx<'_>, menu_source: &str) -> Result<(), String> {
    let js_err = error_renderer(ctx);

    let menu = Module::declare(ctx.clone(), MENU_NAME, menu_source).map_err(js_err)?;
    let (evaluated, promise) = menu.eval().map_err(js_err)?;
    promise.finish::<()>().map_err(js_err)?;

    let default: rquickjs::Value = evaluated.get("default").map_err(js_err)?;
    ctx.globals().set(MENU_GLOBAL, default).map_err(js_err)
}

/// Call `menu.invoke(props)` and return the serialized `Menu`.
fn invoke(ctx: &Ctx<'_>, props_json: &str) -> Result<String, String> {
    let js_err = error_renderer(ctx);

    let globals = ctx.globals();

    // Pull the menu export out first and keep it as the receiver: `This` and the
    // props argument would otherwise both borrow `globals`.
    let menu: rquickjs::Value = globals.get(MENU_GLOBAL).map_err(js_err)?;
    if menu.is_undefined() || menu.is_null() {
        return Err("menu module has no default export".to_string());
    }
    let menu_obj: rquickjs::Object = menu
        .clone()
        .into_object()
        .ok_or_else(|| "menu module default export is not an object".to_string())?;

    // Cross the Rust/JS boundary with the context's own JSON, so the module sees
    // a real object rather than a string.
    let json: rquickjs::Object = globals.get("JSON").map_err(js_err)?;
    let parse: rquickjs::Function = json.get("parse").map_err(js_err)?;
    let stringify: rquickjs::Function = json.get("stringify").map_err(js_err)?;

    let props: rquickjs::Value = parse.call((props_json,)).map_err(js_err)?;
    let invoke: rquickjs::Function = menu_obj.get("invoke").map_err(js_err)?;

    let result: rquickjs::Value = invoke.call((This(menu), props)).map_err(js_err)?;
    stringify.call((result,)).map_err(js_err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::evaluate_once;

    /// The built-ins `rcm-kit` imports are registered by name, which is what makes
    /// `import * as fs from "fs"` resolve.
    #[test]
    fn the_builtin_node_modules_resolve_and_work() {
        let json = evaluate_once::<QuickJsEngine>(
            r#"import * as fs from "fs";
               import * as os from "os";
               import * as path from "path";
               export default { invoke: () => ({
                 iconItems: [],
                 groups: [{
                   key: path.join("a", "b"),
                   label: fs.existsSync(os.homedir()) ? "yes" : "no",
                 }],
               }) };"#,
        )
        .expect("menu evaluation failed");

        let menu: rcm_core::Menu = serde_json::from_str(&json).expect("parse menu");
        assert_eq!(menu.groups[0].label, "yes");
        assert!(menu.groups[0].key.contains('a'));
    }

    /// QuickJS reports the module the error came from, which is the only way to
    /// tell a broken menu from a broken runtime.
    #[test]
    fn a_syntax_error_reports_its_location() {
        let msg = evaluate_once::<QuickJsEngine>("export default { invoke( { } }")
            .expect_err("expected a syntax error");
        assert!(msg.contains("rcm-menu"), "location lost, got: {msg}");
    }
}
