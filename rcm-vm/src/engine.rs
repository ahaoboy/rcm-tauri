//! The abstraction every engine implements, and the process-wide host for it.
//!
//! An engine has two jobs: build one runtime ([`Engine::init`], expensive, once)
//! and run `menu.invoke(props)` on it ([`Engine::evaluate`], cheap, per
//! right-click). Keeping them apart is what makes a click fast — standing up V8
//! or QuickJS costs orders of magnitude more than evaluating a menu.
//!
//! The two halves of a menu are treated differently, because they have different
//! lifetimes:
//!
//! * `rcm-kit` is compiled into the binary. It is evaluated once, with the
//!   runtime, and never again.
//! * `rcm.js` lives next to the executable and is read at runtime, so it is
//!   [`Engine::evaluate`]'s input. Nothing about it is cached: every call gets
//!   the source as it is on disk at that moment.
//!
//! [`EngineHost`] owns the runtime on a thread of its own and hands callers a
//! channel to it. That indirection is not incidental: a `static` must be `Sync`
//! and no JS runtime is (they hold `Rc`s and an isolate), so the host stores only
//! the `Send` half and the engine never leaves its thread.
//!
//! Adding an engine means implementing [`Engine`] and pointing `Current` at it in
//! `lib.rs`; nothing else here or in the public API changes.

use std::sync::mpsc::{Receiver, Sender, SyncSender, channel, sync_channel};

/// A JavaScript runtime that can evaluate the menu module.
pub(crate) trait Engine: Sized {
    /// Build the runtime and load `menu_source` into it.
    ///
    /// The menu is loaded here rather than on first use so that a failed menu is
    /// reported at startup, and so the warm-up (`rcm-kit`, the Node polyfills
    /// behind it) happens off the first click's critical path.
    fn init(menu_source: &str) -> Result<Self, String>;

    /// Run `menu_source`'s `menu.invoke(props)` and return the serialized `Menu`.
    ///
    /// `menu_source` is read from disk for every call and is what runs, whether
    /// or not it differs from the previous one.
    fn evaluate(&mut self, menu_source: &str, props_json: &str) -> Result<String, String>;
}

/// One request: props JSON in, `Menu` JSON out.
type Job = (String, Sender<Result<String, String>>);

/// The channel to the engine, which lives on the thread that built it.
pub(crate) struct EngineHost {
    jobs: Option<Sender<Job>>,
    /// Why the engine never came up, if it did not.
    startup_error: Option<String>,
}

impl EngineHost {
    /// Set up engine `E` on its own thread and wait for it to be ready.
    pub(crate) fn init<E: Engine>() -> Self {
        let (jobs_tx, jobs_rx) = channel::<Job>();
        let (ready_tx, ready_rx) = sync_channel::<Result<(), String>>(1);

        let spawned = std::thread::Builder::new()
            .name("rcm-vm".to_string())
            .spawn(move || serve::<E>(jobs_rx, ready_tx));

        if let Err(e) = spawned {
            return Self::failed(format!("spawn rcm-vm thread: {e}"));
        }

        // Callers would block on the first build anyway, so it happens before
        // this returns: a failure is then reported once instead of per call.
        match ready_rx.recv() {
            Ok(Ok(())) => Self {
                jobs: Some(jobs_tx),
                startup_error: None,
            },
            Ok(Err(e)) => Self::failed(e),
            Err(_) => Self::failed("rcm-vm thread died before it was ready".to_string()),
        }
    }

    fn failed(error: String) -> Self {
        Self {
            jobs: None,
            startup_error: Some(error),
        }
    }

    /// Run one evaluation on the engine thread.
    pub(crate) fn evaluate(&self, props_json: &str) -> Result<String, String> {
        let Some(jobs) = &self.jobs else {
            return Err(self
                .startup_error
                .clone()
                .unwrap_or_else(|| "rcm-vm engine is not running".to_string()));
        };

        let (reply_tx, reply_rx) = channel();
        jobs.send((props_json.to_string(), reply_tx))
            .map_err(|_| "rcm-vm engine stopped".to_string())?;
        reply_rx
            .recv()
            .map_err(|_| "rcm-vm engine dropped the request".to_string())?
    }
}

/// The host thread: build once, then serve until the channel closes.
fn serve<E: Engine>(jobs: Receiver<Job>, ready: SyncSender<Result<(), String>>) {
    // The menu is read here, on every click, so an edit to `rcm.js` is picked up
    // without anything having to notice it changed.
    let mut engine = match E::init(&rcm_core::menu::load_menu_module()) {
        Ok(engine) => {
            let _ = ready.send(Ok(()));
            engine
        }
        Err(e) => {
            let _ = ready.send(Err(e));
            return;
        }
    };

    for (props_json, reply) in jobs {
        let menu = rcm_core::menu::load_menu_module();
        let _ = reply.send(engine.evaluate(&menu, &props_json));
    }
}

// ── Test fixtures ──────────────────────────────────────────────────────────
// Shared, so every engine is put through the same requests.

/// The props every engine test evaluates with.
#[cfg(test)]
pub(crate) fn test_props() -> rcm_core::InvokeProps {
    rcm_core::InvokeProps {
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
    }
}

#[cfg(test)]
pub(crate) fn props_json() -> String {
    serde_json::to_string(&test_props()).expect("serialize props")
}

/// Build engine `E` on `menu_source` and evaluate it once.
#[cfg(test)]
pub(crate) fn evaluate_once<E: Engine>(menu_source: &str) -> Result<String, String> {
    let mut engine = E::init(menu_source)?;
    engine.evaluate(menu_source, &props_json())
}

/// Builds engine `E` on the first source, then evaluates each source in turn —
/// letting a test see whether the menu module was evaluated again.
#[cfg(test)]
fn keys<E: Engine>(menu_sources: &[&str]) -> Vec<String> {
    let first = menu_sources.first().expect("at least one menu");
    let mut engine = E::init(first).expect("init engine");
    menu_sources
        .iter()
        .map(|source| {
            let json = engine.evaluate(source, &props_json()).expect("evaluate");
            menu_key(&json)
        })
        .collect()
}

/// The `key` of the first group in a serialized menu.
#[cfg(test)]
fn menu_key(json: &str) -> String {
    let menu: crate::Menu = serde_json::from_str(json).expect("parse menu");
    menu.groups[0].key.clone()
}

/// A menu whose group key is how many times the runtime has evaluated a menu.
///
/// The count lives in a global, so it survives the menu being replaced — but not
/// the runtime being rebuilt, which is what lets a test tell the two apart.
#[cfg(test)]
const COUNTING_MENU: &str = r#"globalThis.__runs = (globalThis.__runs ?? 0) + 1;
export default {
  invoke: () => ({ iconItems: [], groups: [{ key: String(globalThis.__runs) }] }),
};"#;

/// [`COUNTING_MENU`] after an edit: the key changes, which is what says the edited
/// source ran. Stands in for a user changing `rcm.js`.
#[cfg(test)]
const EDITED_MENU: &str = r#"export default {
  invoke: () => ({ iconItems: [], groups: [{ key: "edited" }] }),
};"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Current;

    /// Rebuilding the runtime per click is what made right-clicks slow, so
    /// repeated calls must share one runtime. The count climbing is what proves
    /// it did — a fresh runtime would restart it. Runs against whichever engine is
    /// compiled in.
    #[test]
    fn evaluations_reuse_the_runtime() {
        let runs: Vec<u64> = keys::<Current>(&[COUNTING_MENU, COUNTING_MENU, COUNTING_MENU])
            .iter()
            .map(|key| key.parse().expect("a run count"))
            .collect();
        assert!(
            runs.windows(2).all(|pair| pair[1] == pair[0] + 1),
            "the runtime restarted between calls: {runs:?}"
        );
    }

    /// The menu is read from disk on every call, so the call after an edit has to
    /// run the edited source rather than the one already compiled.
    #[test]
    fn the_newest_menu_always_runs() {
        let keys = keys::<Current>(&[COUNTING_MENU, EDITED_MENU, EDITED_MENU]);
        assert_eq!(&keys[1..], ["edited", "edited"]);
    }

    /// The frontend calls `invoke` from `tauri::async_runtime` (tokio), so an
    /// engine that borrowed the caller's runtime used to panic. The host owning
    /// its own thread is what makes this safe.
    #[tokio::test]
    async fn the_host_can_be_used_from_an_async_runtime() {
        let host = EngineHost::init::<Current>();
        // The point is that this returns at all rather than panicking.
        host.evaluate(&props_json()).expect("evaluation");
    }

    /// A thrown error has to reach the caller with its message intact — the
    /// frontend only ever sees the string.
    #[test]
    fn a_thrown_error_reports_its_message() {
        let msg =
            evaluate_once::<Current>("export default { invoke() { throw new Error('boom') } }")
                .expect_err("expected the menu to fail");
        assert!(msg.contains("boom"), "message lost, got: {msg}");
    }

    /// A menu without `invoke` is a user error, and has to say so.
    #[test]
    fn a_missing_invoke_export_is_reported() {
        let msg =
            evaluate_once::<Current>("export default {}").expect_err("expected the menu to fail");
        assert!(msg.contains("invoke"), "message lost, got: {msg}");
    }

    /// `rcm-kit` logs through a bare `print`, which is a QuickJS global rather
    /// than a Deno one, so each engine has to supply it. Nearly every code path
    /// that builds a menu item calls it.
    #[test]
    fn print_is_available_to_the_menu() {
        evaluate_once::<Current>(
            r#"export default { invoke: () => { print("hello"); return { iconItems: [], groups: [] } } };"#,
        )
        .expect("menu evaluation failed");
    }
}
