//! `@trash` — Move file(s) to the recycle bin.

use crate::rcm::CommandPayload;
use super::{SystemCmdResult, powershell};

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let paths: Vec<&str> = cmd.args.iter().map(|s| s.as_str()).collect();
    if paths.is_empty() {
        return SystemCmdResult { success: false, message: "No files specified".into() };
    }

    let quoted = paths
        .iter()
        .map(|p| format!("'{p}'"))
        .collect::<Vec<_>>()
        .join(", ");

    let script = format!(
        "$shell = New-Object -ComObject Shell.Application; \
         $items = @({quoted}); \
         foreach ($item in $items) {{ \
           $shell.Namespace(0).ParseName((Get-Item $item).Name).InvokeVerb('delete') \
         }}"
    );
    match powershell(&script) {
        Ok(_) => SystemCmdResult { success: true, message: "Moved to recycle bin".into() },
        Err(e) => SystemCmdResult { success: false, message: e },
    }
}
