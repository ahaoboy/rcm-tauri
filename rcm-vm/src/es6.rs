//! The in-tree `es6` engine — selected by the `es6` feature.
//!
//! `es6` provides the ECMAScript runtime, `Promise`/job loop, host modules
//! (`fs`, `os`, `path`, `console`, `timers`) and a native ES module API
//! (`Ctx::declare_module` / `Module::eval`). Modules are declared from source
//! and evaluated to an exports namespace, so this backend reads exports
//! directly — the same shape as the QuickJS backend:
//!
//! * `rcm-kit` is declared and evaluated once; its namespace is registered
//!   under [`LIB_NAME`] for the menu's imports.
//! * The menu is declared and evaluated on every call; its `default` export
//!   is the object with `invoke`.
//!
//! Imports, `export`, `export default`, `export { x as default }` and private
//! class elements (`#name`) are all handled by the engine itself; the backend
//! only runs modules and reads their exports.

use es6::{Context, Ctx, JsValue, Runtime, Value};

use crate::engine::Engine;
use crate::{LIB_MODULE, LIB_NAME, MENU_NAME};

/// A live `es6` context with `rcm-kit` evaluated and registered.
pub(crate) struct Es6Engine {
    ctx: Context,
}

impl Engine for Es6Engine {
    const NAME: &'static str = "es6";
    const VERSION: &'static str = es6::VERSION_STRING;
    const COMMIT: &'static str = es6::GIT_COMMIT_HASH;
    const REPO: &'static str = "https://github.com/ahaoboy/es6";

    fn init(menu_source: &str) -> Result<Self, String> {
        let runtime = Runtime::new().map_err(|e| e.to_string())?;
        let ctx = runtime.context();

        let print = ctx.function("print", 1, |realm, _this, arguments| {
            let text = format(realm, arguments)?;
            println!("{text}");
            Ok(JsValue::Undefined)
        });
        ctx.global()
            .set("print", JsValue::from_function(print.to_ref()));

        bundle(&ctx)?;
        // The warm-up's result is discarded, like the LLRT backend: a broken
        // menu is reported by the call that needs it.
        ctx.with(|ctx| {
            let _ = load_menu(ctx, menu_source);
        });
        Ok(Self { ctx })
    }

    fn evaluate(&mut self, menu_source: &str, props_json: &str) -> Result<String, String> {
        self.ctx.with(|ctx| {
            let menu = load_menu(ctx, menu_source)?;
            invoke(ctx, &menu, props_json)
        })
    }
}

/// Render an engine error with its source position when the engine recorded
/// one, so a broken module is reported by line rather than by message alone.
fn describe(error: es6::Error) -> String {
    match (error.line(), error.column()) {
        (Some(line), Some(column)) => format!("{error} at {line}:{column}"),
        _ => error.to_string(),
    }
}

fn format(realm: &mut es6::runtime::Realm, values: &[JsValue]) -> Result<String, JsValue> {
    let mut parts = Vec::with_capacity(values.len());
    for value in values {
        parts.push(es6::vm::to_string(realm, value.clone())?);
    }
    Ok(parts.join(" "))
}

/// Evaluate `rcm-kit` once and register its exports under `rcm-kit`.
fn bundle(ctx: &Context) -> Result<(), String> {
    let exports = ctx
        .eval_module(LIB_NAME, LIB_MODULE)
        .map_err(|e| format!("{LIB_NAME}: {}", describe(e)))?;
    ctx.register_module(LIB_NAME, JsValue::Object(exports.into_ref()));
    Ok(())
}

/// Declare and evaluate the menu module, returning its default export.
fn load_menu<'js>(ctx: Ctx<'js>, source: &str) -> Result<Value<'js>, String> {
    let exports = ctx
        .declare_module(MENU_NAME, source)
        .and_then(|module| module.eval())
        .map_err(|e| format!("{MENU_NAME}: {}", describe(e)))?;
    let default = exports
        .get_value("default")
        .map_err(|e| format!("{MENU_NAME}: {}", describe(e)))?;
    if default.is_nullish() {
        return Err(format!("{MENU_NAME}: no default export"));
    }
    Ok(default)
}

/// Call `menu.invoke(props)` and serialize the result.
fn invoke(ctx: Ctx<'_>, menu: &Value<'_>, props_json: &str) -> Result<String, String> {
    let object = menu
        .as_object()
        .ok_or_else(|| format!("{MENU_NAME}: default export is not an object"))?;
    let invoke = object
        .get_value("invoke")
        .map_err(|e| format!("{MENU_NAME}: {}", describe(e)))?;
    let Some(function) = invoke.as_function() else {
        return Err(format!("{MENU_NAME}: menu module has no invoke() export"));
    };
    let props = ctx
        .parse_json(props_json)
        .map_err(|e| format!("{MENU_NAME}: {}", describe(e)))?;
    let result: JsValue = function
        .call_with_this(menu, std::slice::from_ref(&props))
        .map_err(|e| format!("{MENU_NAME}: {}", describe(e)))?;
    let result = ctx.value(result);
    ctx.stringify_json(&result)
        .map_err(|e| format!("{MENU_NAME}: {}", describe(e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::evaluate_once;

    #[test]
    fn the_builtin_node_modules_resolve_and_work() {
        let json = evaluate_once::<Es6Engine>(
            r#"import * as fs from "fs";
               import * as os from "os";
               import * as path from "path";
               export default { invoke: () => ({
                 iconItems: [],
                 groups: [{
                   key: path.join("a", "b"),
                   label: fs.readdirSync(os.homedir()).length > 0 ? "yes" : "no",
                 }],
               }) };"#,
        )
        .expect("menu evaluation failed");

        let menu: rcm_core::Menu = serde_json::from_str(&json).expect("parse menu");
        assert_eq!(menu.groups[0].label, "yes");
        assert!(menu.groups[0].key.contains('a'));
    }

    #[test]
    fn a_syntax_error_reports_its_location() {
        let msg = evaluate_once::<Es6Engine>("export default { invoke( { } }")
            .expect_err("expected a syntax error");
        assert!(msg.contains("rcm-menu"), "location lost, got: {msg}");
    }
}
