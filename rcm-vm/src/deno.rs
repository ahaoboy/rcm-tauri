//! The V8 engine (via `deno_runtime`), selected by the `deno` feature.
//!
//! `deno_runtime` is the whole Deno runtime rather than raw V8, so menu code and
//! the bundled `rcm-kit` get `node:fs`, `node:os`, `node:path` and Deno's other
//! built-ins. No Node compatibility is implemented here — it all comes from
//! `deno_node` inside the runtime.
//!
//! Modules are served from memory: [`StaticModuleLoader`] holds the bundled
//! `rcm-kit` runtime, while [`RcmLoader`] maps the *bare* specifiers used by the
//! menu and the bundle (`rcm-kit`, `fs`, `node:fs`, …) onto it and onto the
//! runtime's `node:` modules. The standard resolver only understands URLs and
//! relative paths, so without that mapping `import { … } from "rcm-kit"` would
//! not resolve.
//!
//! The menu is not one of those modules. It is read from disk and compiled
//! straight from source on every call, so the newest `rcm.js` is always what
//! runs — see [`evaluate_menu`].
//!
//! The shape an engine has to have — build once, evaluate per call — is
//! [`crate::engine::Engine`]; this module is only the Deno-specific half.

use std::rc::Rc;
use std::sync::Arc;

use deno_core::{
    ModuleLoadOptions, ModuleLoadReferrer, ModuleLoadResponse, ModuleLoader, ModuleResolveResponse,
    ModuleSpecifier, PollEventLoopOptions, ResolutionKind, StaticModuleLoader, serde_v8, v8,
};
use deno_fs::RealFs;
use deno_permissions::PermissionsContainer;
use deno_resolver::npm::{DenoInNpmPackageChecker, NpmResolver};
use deno_runtime::deno_node::is_builtin_node_module;
use deno_runtime::deno_web::BlobStore;
use deno_runtime::permissions::RuntimePermissionDescriptorParser;
use deno_runtime::worker::{MainWorker, WorkerOptions, WorkerServiceOptions};
use sys_traits::impls::RealSys;

use crate::engine::Engine;
use crate::{LIB_MODULE, LIB_NAME, MENU_NAME};

/// Specifiers the in-memory modules are addressed by.
const KIT_URL: &str = "file:///rcm-kit.js";
const MENU_URL: &str = "file:///rcm-menu.js";
/// The worker's nominal main module. Nothing loads it — the menu is compiled
/// from source instead — so it only fills in the runtime's entry-point
/// bookkeeping.
const MAIN_URL: &str = "file:///rcm-main.js";

/// The script the runtime is seeded with: `rcm-kit` logs through a bare
/// `print`, which is a QuickJS/LLRT global rather than a Deno one.
const PRINT_SHIM: &str = "globalThis.print = (text) => {\n\
                          \x20 const line = String(text);\n\
                          \x20 if (globalThis.Deno?.core?.print) globalThis.Deno.core.print(line + \"\\n\");\n\
                          \x20 else console.log(line);\n\
                          };\n";

/// Parse one of the compile-time-constant specifiers above.
fn parse_specifier(url: &str) -> Result<ModuleSpecifier, String> {
    ModuleSpecifier::parse(url).map_err(|e| format!("invalid module specifier {url}: {e}"))
}

/// Rewrite a Node built-in specifier so the runtime's `node:` modules serve it.
///
/// The bundled `rcm-kit` imports built-ins the Node way (`fs`, `os`, `path`),
/// while Deno only publishes them under `node:`. Both spellings are normalised
/// here so menu code can use whichever it prefers.
fn node_builtin(specifier: &str) -> Option<ModuleSpecifier> {
    let name = specifier.strip_prefix("node:").unwrap_or(specifier);
    (specifier.starts_with("node:") || is_builtin_node_module(name))
        .then(|| parse_specifier(&format!("node:{name}")).ok())
        .flatten()
}

/// Serves `rcm-kit` from memory and resolves the bare specifiers the menu uses.
///
/// The standard resolver only understands URLs and relative paths, so without
/// the `rcm-kit` mapping `import { newMenu } from "rcm-kit"` would not resolve.
/// `node:` built-ins and relative paths need no help — it already parses those
/// as URLs.
///
/// The menu is deliberately absent: it is compiled from source on every call, so
/// there is nothing about it to hold here.
struct RcmLoader {
    inner: StaticModuleLoader,
    kit: ModuleSpecifier,
    menu: ModuleSpecifier,
}

impl RcmLoader {
    fn new() -> Result<Self, String> {
        let kit = parse_specifier(KIT_URL)?;
        let inner = StaticModuleLoader::new([(kit.clone(), LIB_MODULE.to_string())]);
        Ok(Self {
            inner,
            kit,
            menu: parse_specifier(MENU_URL)?,
        })
    }
}

impl ModuleLoader for RcmLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        kind: ResolutionKind,
    ) -> ModuleResolveResponse {
        match specifier {
            LIB_NAME => Ok(self.kit.clone()),
            MENU_NAME => Ok(self.menu.clone()),
            _ => match node_builtin(specifier) {
                Some(builtin) => Ok(builtin),
                None => self.inner.resolve(specifier, referrer, kind),
            },
        }
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        maybe_referrer: Option<&ModuleLoadReferrer>,
        options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
        self.inner.load(module_specifier, maybe_referrer, options)
    }
}

/// A live Deno runtime and the tokio runtime that drives it.
///
/// The runtime is built once and never rebuilt, which is what keeps `rcm-kit`
/// evaluated exactly once. The loader is not held here — the worker keeps it
/// alive — and it carries no per-call state, since the menu is compiled from
/// source rather than served from it.
///
/// Both runtimes live on the host thread and have to be created, used and dropped
/// together — `deno_runtime` stores tokio handles in its op state. The field order
/// is deliberate: Rust drops fields in declaration order, so the worker goes
/// before the runtime it depends on.
pub(crate) struct DenoEngine {
    worker: MainWorker,
    rt: tokio::runtime::Runtime,
}

impl Engine for DenoEngine {
    fn init(menu_source: &str) -> Result<Self, String> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("tokio runtime: {e}"))?;

        let worker = {
            let _entered = rt.enter();
            bootstrap(Rc::new(RcmLoader::new()?))?
        };

        let mut engine = Self { worker, rt };
        let Self { worker, rt } = &mut engine;
        // Warm-up. A failure is ignored: a broken menu is reported by the call
        // that needs it.
        let _ = rt.block_on(evaluate_menu(worker, menu_source));
        Ok(engine)
    }

    fn evaluate(&mut self, menu_source: &str, props_json: &str) -> Result<String, String> {
        let Self { worker, rt } = self;
        rt.block_on(invoke_menu(worker, menu_source, props_json))
    }
}

/// Build the runtime and seed it with the `print` shim.
fn bootstrap(loader: Rc<RcmLoader>) -> Result<MainWorker, String> {
    // A menu is trusted, locally installed code, so it runs with full access.
    let permissions =
        PermissionsContainer::allow_all(Arc::new(RuntimePermissionDescriptorParser::new(RealSys)));

    // `node_services: None` leaves the Node extension itself in place (its
    // `node:` built-ins work) while skipping npm/`node_modules` resolution.
    let services: WorkerServiceOptions<DenoInNpmPackageChecker, NpmResolver<RealSys>, RealSys> =
        WorkerServiceOptions {
            module_loader: loader,
            permissions,
            blob_store: Arc::new(BlobStore::default()),
            broadcast_channel: Default::default(),
            feature_checker: Default::default(),
            node_services: None,
            npm_process_state_provider: None,
            root_cert_store_provider: None,
            fetch_dns_resolver: Default::default(),
            shared_array_buffer_store: None,
            compiled_wasm_module_store: None,
            v8_code_cache: None,
            fs: Arc::new(RealFs),
            bundle_provider: None,
            deno_rt_native_addon_loader: None,
        };

    let main = parse_specifier(MAIN_URL)?;
    let mut worker = MainWorker::bootstrap_from_options::<
        DenoInNpmPackageChecker,
        NpmResolver<RealSys>,
        RealSys,
    >(&main, services, WorkerOptions::default());

    worker
        .js_runtime
        .execute_script("rcm-print", PRINT_SHIM)
        .map_err(|e| e.to_string())?;

    Ok(worker)
}

/// Compile `menu_source` under [`MENU_URL`] and evaluate it, so that the next
/// `import(RCM_MENU)` hands back this module.
///
/// Nothing about the menu is cached, on either side: the source is whatever the
/// caller read from disk, and each call compiles it afresh under the same name,
/// which is the only way to get a runtime to evaluate a module it already has.
/// Imports of `rcm-kit` inside it still find the module evaluated at startup.
async fn evaluate_menu(worker: &mut MainWorker, menu_source: &str) -> Result<(), String> {
    let js_runtime = &mut worker.js_runtime;
    let specifier = parse_specifier(MENU_URL)?;

    let id = js_runtime
        .load_side_es_module_from_code(&specifier, menu_source.to_string())
        .await
        .map_err(|e| e.to_string())?;

    js_runtime.mod_evaluate(id).await.map_err(|e| e.to_string())
}

/// Run `menu.invoke(props)` on the most recently evaluated menu and return the
/// serialized `Menu`.
async fn invoke_menu(
    worker: &mut MainWorker,
    menu_source: &str,
    props_json: &str,
) -> Result<String, String> {
    // Loading the menu here is what makes every call run the newest source, and
    // is also what warms the runtime at startup: the first load pulls in
    // `rcm-kit` and the lazily-loaded Node polyfills behind `node:fs`, which
    // costs rather more than a warm call. A failure is ignored there and
    // reported by the call that needs the menu.
    evaluate_menu(worker, menu_source).await?;

    // `props` travels as a JSON literal parsed by JS, which keeps the escaping
    // out of the generated source.
    let literal = serde_json::to_string(props_json).map_err(|e| e.to_string())?;
    let script = format!(
        "(async () => JSON.stringify(\
         (await import(\"{MENU_NAME}\")).default.invoke(JSON.parse({literal}))))()"
    );

    let resolved = resolve_script(worker, "rcm-invoke", script).await?;

    deno_core::scope!(scope, &mut worker.js_runtime);
    let value = v8::Local::new(scope, resolved);
    serde_v8::from_v8(scope, value).map_err(|e| e.to_string())
}

/// Run `script` and resolve the promise it returns.
async fn resolve_script(
    worker: &mut MainWorker,
    name: &'static str,
    script: String,
) -> Result<v8::Global<v8::Value>, String> {
    let promise = worker
        .js_runtime
        .execute_script(name, script)
        .map_err(|e| e.to_string())?;

    let pending = Box::pin(worker.js_runtime.resolve(promise));
    worker
        .js_runtime
        .with_event_loop_promise(pending, PollEventLoopOptions::default())
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::evaluate_once;

    /// The bundled `rcm-kit` imports the Node built-ins, which only resolve once
    /// the runtime supplies its `node:` modules and [`RcmLoader`] re-points the
    /// bare spellings at them.
    #[test]
    fn the_builtin_node_modules_resolve_and_work() {
        let json = evaluate_once::<DenoEngine>(
            r#"import * as fs from "fs";
               import * as os from "os";
               import * as path from "path";
               export default { invoke: () => ({
                 iconItems: [],
                 groups: [{
                   key: path.join("a", "b"),
                   label: fs.existsSync(os.homedir()) ? os.platform() : "missing",
                 }],
               }) };"#,
        )
        .expect("menu evaluation failed");

        let menu: rcm_core::Menu = serde_json::from_str(&json).expect("parse menu");
        assert_eq!(menu.groups[0].key, "a\\b");
        // `os.platform()` is Node's spelling, where Windows is `"win32"`.
        let expected = match std::env::consts::OS {
            "windows" => "win32",
            other => other,
        };
        assert_eq!(menu.groups[0].label, expected);
    }
}
