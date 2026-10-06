//! RCM VM — JavaScript runtime execution engine.
//!
//! Evaluates RCM menu definitions and returns the resulting `Menu`. Two
//! interchangeable engines are available, selected by feature: `deno_runtime`
//! (V8 + Deno/Node built-ins) behind the default `deno`, or `rquickjs` (QuickJS +
//! Node modules) behind `llrt`. Exactly one is compiled, and both expose the same
//! [`invoke`] because both implement [`engine::Engine`].
//!
//! Building a runtime costs far more than evaluating a menu, so the engine is
//! created once and reused; [`init`] exists to pay for that build during startup
//! instead of on the first menu.
//!
//! This crate is framework-agnostic and can be used with any frontend.

#[cfg(all(feature = "deno", feature = "llrt"))]
compile_error!(
    "features `deno` and `llrt` are mutually exclusive: `llrt` extends the rquickjs engine"
);

#[cfg(not(any(feature = "deno", feature = "llrt")))]
compile_error!(
    "no engine selected: enable `deno` (V8 + Deno runtime) or `llrt` (QuickJS + Node modules)"
);

#[cfg(feature = "deno")]
mod deno;
mod engine;
#[cfg(feature = "llrt")]
mod llrt;

/// The engine this build uses. Pointing this at another implementation is the
/// whole of "add an engine".
#[cfg(feature = "deno")]
type Current = deno::DenoEngine;
#[cfg(feature = "llrt")]
type Current = llrt::QuickJsEngine;

use std::sync::LazyLock;

use engine::EngineHost;
use rcm_core::{InvokeProps, Menu};
use rcm_core::{clipboard, lang};

/// The one runtime in the process: built at init, reused by every right-click.
static ENGINE: LazyLock<EngineHost> = LazyLock::new(EngineHost::init::<Current>);

/// Create the engine, eagerly.
///
/// Call this once at startup. Building the runtime is the most expensive thing
/// `invoke` ever does (~1 s for the `deno` engine), and doing it during init
/// keeps it off the first menu's critical path. Later calls have no effect.
pub fn init() {
    let _ = &*ENGINE;
}

/// Evaluate the current menu module and return the `Menu` it produces.
pub fn invoke(props: &InvokeProps) -> std::result::Result<Menu, Box<dyn std::error::Error>> {
    let props_json = serde_json::to_string(props)?;
    let menu_json = ENGINE.evaluate(&props_json)?;
    serde_json::from_str(&menu_json).map_err(Into::into)
}

/// The `rcm-kit` runtime, bundled into the binary.
///
/// This path must stay inside the crate: cargo only ships files under the crate
/// directory, so a path like `../../rcm-kit/dist/index.js` resolves when building
/// from a checkout of this repo but fails for anyone consuming the crate.
///
/// The file is generated from `rcm-kit` by `bun build:rcm`; CI verifies it has
/// not gone stale.
pub(crate) const LIB_MODULE: &str = include_str!("../assets/index.js");

/// Specifier the menu source imports the runtime from
/// (`import { … } from "rcm-kit"`).
pub(crate) const LIB_NAME: &str = "rcm-kit";

/// Specifier the menu source itself is loaded under.
pub(crate) const MENU_NAME: &str = "rcm-menu";

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

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole public path, against the menu the app actually ships — the
    /// engine tests use toy menus and never touch the bundled `rcm-kit`.
    #[test]
    fn invoke_builds_the_bundled_menu() {
        let menu = invoke(&engine::test_props()).expect("invoke");
        assert!(
            !menu.groups.is_empty(),
            "the bundled menu produced no groups"
        );
    }
}
