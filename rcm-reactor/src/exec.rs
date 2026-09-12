//! Fire-and-forget command execution.
//!
//! The shared [`rcm_core::runner`] engine is `async` (it is built on
//! `tokio::process`), so results are driven on a shared background runtime
//! rather than on the Reactor UI thread.

use std::sync::OnceLock;

use rcm_core::CommandPayload;

/// The shared runtime, built once on first use.
///
/// `None` if it could not be built, which only happens under resource
/// exhaustion; commands then fail instead of taking the whole app down.
fn runtime() -> Option<&'static tokio::runtime::Runtime> {
    static RT: OnceLock<Option<tokio::runtime::Runtime>> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .inspect_err(|e| {
                rcm_core::log::error("RcmReactor::execute", &format!("no runtime: {e}"));
            })
            .ok()
    })
    .as_ref()
}

/// Execute a menu command asynchronously, logging any failure.
pub fn execute(cmd: CommandPayload) {
    let Some(runtime) = runtime() else {
        return;
    };
    runtime.spawn(async move {
        rcm_core::runner::execute_logged(&cmd, "RcmReactor::execute").await;
    });
}
