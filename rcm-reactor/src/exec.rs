//! Fire-and-forget command execution.
//!
//! The shared [`rcm_core::runner`] engine is `async` (it is built on
//! `tokio::process`), so results are driven on a shared background runtime
//! rather than on the Reactor UI thread.

use std::sync::OnceLock;

use rcm_core::CommandPayload;

fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime")
    })
}

/// Execute a menu command asynchronously, logging any failure.
pub fn execute(cmd: CommandPayload) {
    runtime().spawn(async move {
        rcm_core::runner::execute_logged(&cmd, "RcmReactor::execute").await;
    });
}
