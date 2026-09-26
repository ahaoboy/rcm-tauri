//! Reusable parsing of positional command arguments.
//!
//! Commands read the payload through [`CmdArgs`], keeping the extraction in one
//! place so every failure names the argument and the expected form. Only the
//! shapes the commands actually use are provided: single string, path, the
//! whole list, and the trailing list.

use std::path::PathBuf;

use super::CmdError;
use crate::types::CommandPayload;

/// Borrowing view over a payload's positional arguments.
pub(crate) struct CmdArgs<'a> {
    raw: &'a [String],
}

impl<'a> CmdArgs<'a> {
    /// Wrap `payload`'s arguments.
    pub(crate) fn of(payload: &'a CommandPayload) -> Self {
        Self { raw: &payload.args }
    }

    /// The first non-empty argument, or `None`.
    pub(crate) fn optional(&self, index: usize) -> Option<&'a str> {
        self.raw
            .get(index)
            .map(String::as_str)
            .filter(|value| !value.is_empty())
    }

    /// The argument at `index`, or a [`CmdError::Missing`] naming it.
    pub(crate) fn required(
        &self,
        index: usize,
        name: &'static str,
        expected: &'static str,
    ) -> Result<&'a str, CmdError> {
        self.optional(index)
            .ok_or_else(|| CmdError::missing(name, expected))
    }

    /// The argument at `index` as a [`PathBuf`].
    pub(crate) fn path(
        &self,
        index: usize,
        name: &'static str,
        expected: &'static str,
    ) -> Result<PathBuf, CmdError> {
        Ok(PathBuf::from(self.required(index, name, expected)?))
    }

    /// The whole (non-empty) argument list, or a [`CmdError::Missing`].
    pub(crate) fn all(
        &self,
        name: &'static str,
        expected: &'static str,
    ) -> Result<&'a [String], CmdError> {
        if self.raw.is_empty() {
            Err(CmdError::missing(name, expected))
        } else {
            Ok(self.raw)
        }
    }

    /// The arguments from `start` onward, e.g. the source list of `@zip`.
    pub(crate) fn tail(&self, start: usize) -> &'a [String] {
        self.raw.get(start..).unwrap_or(&[])
    }
}
