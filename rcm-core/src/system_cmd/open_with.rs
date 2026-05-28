//! `@open-with` — Open the Windows "Open With" dialog.

use crate::types::CommandPayload;
use super::{SystemCmdResult, build_sys_cmd};

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let path = cmd.args.first().cloned().unwrap_or_default();
    match build_sys_cmd("rundll32.exe", &CommandPayload {
        exe: "rundll32.exe".into(),
        args: vec!["shell32.dll,OpenAs_RunDLL".into(), path],
        ..cmd.clone()
    })
    .spawn()
    {
        Ok(_) => SystemCmdResult { success: true, message: "Open With dialog launched".into() },
        Err(e) => SystemCmdResult { success: false, message: format!("OpenWith failed: {e}") },
    }
}
