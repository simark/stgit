// SPDX-License-Identifier: GPL-2.0-only

use std::{cell::RefCell, io::Write};

use anyhow::Result;
use termcolor::WriteColor;

use super::PushStatus;
use crate::patch::PatchName;

/// The annotation shown next to a patch that was pushed, if any.
fn push_status_str(status: PushStatus) -> &'static str {
    match status {
        PushStatus::New => "(new)",
        PushStatus::AlreadyMerged => "(merged)",
        PushStatus::Conflict => "(conflict)",
        PushStatus::Empty => "(empty)",
        PushStatus::Modified => "(modified)",
        PushStatus::Unmodified => "",
    }
}

/// User output for stack transactions.
pub(super) struct TransactionUserInterface {
    output: RefCell<termcolor::StandardStream>,
    printed_top: bool,

    /// Print the resulting series instead of the individual pushed and popped patches.
    ///
    /// From the "stgit.series.show-after-change" configuration variable.
    show_series: bool,

    /// Print the series with the topmost patch first.
    ///
    /// From the "stgit.series.top-first" configuration variable.
    top_first: bool,

    /// Whether the transaction changed which patches are in the series.
    ///
    /// Only relevant when `show_series` is enabled, where it determines whether the
    /// resulting series is worth printing at all.
    series_changed: bool,

    /// Status of each patch pushed by the transaction.
    ///
    /// Only recorded when `show_series` is enabled, so that the statuses may be
    /// annotated onto the series printed at the end of the transaction.
    push_statuses: Vec<(PatchName, PushStatus)>,
}

impl TransactionUserInterface {
    pub(super) fn new(
        output: termcolor::StandardStream,
        show_series: bool,
        top_first: bool,
    ) -> TransactionUserInterface {
        TransactionUserInterface {
            output: RefCell::new(output),
            printed_top: false,
            show_series,
            top_first,
            series_changed: false,
            push_statuses: Vec::new(),
        }
    }

    pub(super) fn printed_top(&self) -> bool {
        // The topmost patch is part of the series printed at the end of the
        // transaction, so it must not be printed on its own as well.
        self.printed_top || self.shows_series()
    }

    /// Whether the resulting series should be printed by [`Self::print_series()`].
    pub(super) fn shows_series(&self) -> bool {
        self.show_series && self.series_changed
    }

    pub(super) fn print_merged(&self, merged_patches: &[&PatchName]) -> Result<()> {
        let mut output = self.output.borrow_mut();
        write!(output, "Found ")?;
        let mut color_spec = termcolor::ColorSpec::new();
        output.set_color(color_spec.set_fg(Some(termcolor::Color::Blue)))?;
        write!(output, "{}", merged_patches.len())?;
        output.reset()?;
        let plural = if merged_patches.len() == 1 { "" } else { "es" };
        writeln!(output, " patch{plural} merged upstream")?;
        Ok(())
    }

    pub(super) fn print_rename(
        &self,
        old_patchname: &PatchName,
        new_patchname: &PatchName,
    ) -> Result<()> {
        let mut output = self.output.borrow_mut();
        let mut color_spec = termcolor::ColorSpec::new();
        output.set_color(color_spec.set_dimmed(true))?;
        write!(output, "{old_patchname}")?;
        color_spec.clear();
        output.set_color(color_spec.set_fg(Some(termcolor::Color::Blue)))?;
        write!(output, " => ")?;
        output.reset()?;
        writeln!(output, "{new_patchname}")?;
        Ok(())
    }

    pub(super) fn print_committed(&mut self, committed: &[PatchName]) -> Result<()> {
        self.series_changed = true;
        let mut output = self.output.borrow_mut();
        let mut color_spec = termcolor::ColorSpec::new();
        output.set_color(color_spec.set_fg(Some(termcolor::Color::Yellow)))?;
        write!(output, "$ ")?;
        color_spec.set_fg(None);
        output.set_color(color_spec.set_intense(true))?;
        write!(output, "{}", committed[0])?;
        if committed.len() > 1 {
            output.set_color(color_spec.set_intense(false))?;
            write!(output, "..")?;
            output.set_color(color_spec.set_intense(true))?;
            let last = &committed[committed.len() - 1];
            write!(output, "{last}")?;
        }
        output.reset()?;
        writeln!(output)?;
        Ok(())
    }

    pub(super) fn print_uncommitted(&mut self, uncommitted: &[PatchName]) -> Result<()> {
        if !uncommitted.is_empty() {
            self.series_changed = true;
            let mut output = self.output.borrow_mut();
            let mut color_spec = termcolor::ColorSpec::new();
            output.set_color(color_spec.set_fg(Some(termcolor::Color::Green)))?;
            write!(output, "+ ")?;
            output.set_color(color_spec.set_fg(None))?;
            let first = uncommitted.first().unwrap();
            if uncommitted.len() == 1 {
                writeln!(output, "{first}")?;
            } else {
                let last = uncommitted.last().unwrap();
                writeln!(output, "{first}..{last}")?;
            }
        }
        Ok(())
    }

    pub(super) fn print_deleted(&mut self, deleted: &[PatchName]) -> Result<()> {
        if !deleted.is_empty() {
            self.series_changed = true;
            let mut output = self.output.borrow_mut();
            let mut color_spec = termcolor::ColorSpec::new();
            output.set_color(color_spec.set_fg(Some(termcolor::Color::Yellow)))?;
            write!(output, "# ")?;
            color_spec.set_fg(None);
            output.set_color(color_spec.set_dimmed(true))?;
            write!(output, "{}", deleted[0])?;
            if deleted.len() > 1 {
                output.set_color(color_spec.set_dimmed(false))?;
                write!(output, "..")?;
                output.set_color(color_spec.set_dimmed(true))?;
                let last = &deleted[deleted.len() - 1];
                write!(output, "{last}")?;
            }
            output.reset()?;
            writeln!(output)?;
        }
        Ok(())
    }

    pub(super) fn print_hidden(&mut self, hidden: &[PatchName]) -> Result<()> {
        self.series_changed = true;
        let mut output = self.output.borrow_mut();
        let mut color_spec = termcolor::ColorSpec::new();
        for patchname in hidden {
            output.set_color(color_spec.set_fg(Some(termcolor::Color::Red)))?;
            write!(output, "! ")?;
            color_spec.set_fg(None);
            output.set_color(color_spec.set_dimmed(true).set_italic(true))?;
            writeln!(output, "{patchname}")?;
            color_spec.clear();
            output.reset()?;
        }
        Ok(())
    }

    pub(super) fn print_unhidden(&mut self, unhidden: &[PatchName]) -> Result<()> {
        self.series_changed = true;
        let mut output = self.output.borrow_mut();
        let mut color_spec = termcolor::ColorSpec::new();
        for patchname in unhidden {
            output.set_color(color_spec.set_fg(Some(termcolor::Color::Magenta)))?;
            write!(output, "- ")?;
            color_spec.set_fg(None);
            output.set_color(color_spec.set_dimmed(true))?;
            writeln!(output, "{patchname}")?;
            color_spec.clear();
            output.reset()?;
        }
        Ok(())
    }

    pub(super) fn print_popped(&mut self, popped: &[PatchName]) -> Result<()> {
        if !popped.is_empty() {
            self.series_changed = true;
            if self.show_series {
                return Ok(());
            }
            let mut output = self.output.borrow_mut();
            let mut color_spec = termcolor::ColorSpec::new();
            output.set_color(color_spec.set_fg(Some(termcolor::Color::Magenta)))?;
            write!(output, "- ")?;
            color_spec.set_fg(None);
            output.set_color(color_spec.set_dimmed(true))?;
            write!(output, "{}", popped[0])?;
            if popped.len() > 1 {
                output.set_color(color_spec.set_dimmed(false))?;
                write!(output, "..")?;
                output.set_color(color_spec.set_dimmed(true))?;
                let last = &popped[popped.len() - 1];
                write!(output, "{last}")?;
            }
            output.reset()?;
            writeln!(output)?;
        }
        Ok(())
    }

    pub(super) fn print_pushed(
        &mut self,
        patchname: &PatchName,
        status: PushStatus,
        is_last: bool,
    ) -> Result<()> {
        self.series_changed = true;
        if self.show_series {
            self.push_statuses.push((patchname.clone(), status));
            return Ok(());
        }
        let mut output = self.output.borrow_mut();
        let sigil = if is_last { '>' } else { '+' };
        let mut color_spec = termcolor::ColorSpec::new();
        output.set_color(
            color_spec.set_fg(Some(if let PushStatus::Conflict = status {
                termcolor::Color::Red
            } else if is_last {
                termcolor::Color::Blue
            } else {
                termcolor::Color::Green
            })),
        )?;
        write!(output, "{sigil} ")?;
        color_spec.clear();
        output.set_color(color_spec.set_bold(is_last).set_intense(!is_last))?;
        write!(output, "{patchname}")?;
        output.reset()?;

        let status_str = push_status_str(status);

        if status_str.is_empty() {
            writeln!(output)?;
        } else {
            color_spec.clear();
            output.set_color(color_spec.set_fg(Some(termcolor::Color::Yellow)))?;
            writeln!(output, " {status_str}")?;
            output.reset()?;
        }
        if is_last {
            self.printed_top = true;
        }
        Ok(())
    }

    /// Print the whole series resulting from the transaction.
    ///
    /// This takes the place of the per-patch output of [`Self::print_pushed()`] and
    /// [`Self::print_popped()`] when the "stgit.series.show-after-change" configuration
    /// variable is enabled. The output matches `stg series`, except that patches pushed
    /// by the transaction retain their push status annotation.
    ///
    /// Hidden patches are not shown, matching the default `stg series` output.
    pub(super) fn print_series(
        &self,
        applied: &[PatchName],
        unapplied: &[PatchName],
    ) -> Result<()> {
        let mut entries: Vec<(&PatchName, char)> =
            Vec::with_capacity(applied.len() + unapplied.len());
        if let Some((top_patchname, rest)) = applied.split_last() {
            entries.extend(rest.iter().map(|patchname| (patchname, '+')));
            entries.push((top_patchname, '>'));
        }
        entries.extend(unapplied.iter().map(|patchname| (patchname, '-')));
        if self.top_first {
            entries.reverse();
        }

        let mut output = self.output.borrow_mut();
        let mut color_spec = termcolor::ColorSpec::new();

        for (patchname, sigil) in entries {
            let status = self
                .push_statuses
                .iter()
                .find(|(pushed_patchname, _)| pushed_patchname == patchname)
                .map(|(_, status)| *status);

            let sigil_color = if let Some(PushStatus::Conflict) = status {
                termcolor::Color::Red
            } else {
                match sigil {
                    '>' => termcolor::Color::Blue,
                    '-' => termcolor::Color::Magenta,
                    _ => termcolor::Color::Green,
                }
            };
            output.set_color(color_spec.set_fg(Some(sigil_color)))?;
            write!(output, "{sigil} ")?;
            color_spec.clear();

            output.set_color(match sigil {
                '>' => color_spec.set_bold(true),
                '-' => color_spec.set_dimmed(true),
                _ => color_spec.set_intense(true),
            })?;
            write!(output, "{patchname}")?;
            output.reset()?;
            color_spec.clear();

            let status_str = status.map_or("", push_status_str);
            if status_str.is_empty() {
                writeln!(output)?;
            } else {
                output.set_color(color_spec.set_fg(Some(termcolor::Color::Yellow)))?;
                writeln!(output, " {status_str}")?;
                output.reset()?;
                color_spec.clear();
            }
        }
        Ok(())
    }

    pub(super) fn print_top(&self, patchname: &PatchName) -> Result<()> {
        let mut output = self.output.borrow_mut();
        let mut color_spec = termcolor::ColorSpec::new();
        output.set_color(color_spec.set_fg(Some(termcolor::Color::Blue)))?;
        write!(output, "> ")?;
        color_spec.clear();
        output.set_color(color_spec.set_bold(true).set_intense(false))?;
        writeln!(output, "{patchname}")?;
        output.reset()?;
        Ok(())
    }

    pub(super) fn print_rolled_back(&self, patchname: Option<&PatchName>) -> Result<()> {
        let mut output = self.output.borrow_mut();
        let mut color_spec = termcolor::ColorSpec::new();
        output.set_color(color_spec.set_fg(Some(termcolor::Color::Blue)))?;
        write!(output, "@ ")?;
        color_spec.clear();
        output.set_color(color_spec.set_bold(true).set_intense(false))?;
        if let Some(patchname) = patchname {
            write!(output, "{patchname}")?;
        } else {
            write!(output, "{{base}}")?;
        }
        output.reset()?;
        writeln!(output, " (rolled back)")?;
        Ok(())
    }

    pub(super) fn print_exec(&self, cmd: &str) -> Result<()> {
        let mut output = self.output.borrow_mut();
        let mut color_spec = termcolor::ColorSpec::new();
        output.set_color(color_spec.set_fg(Some(termcolor::Color::Yellow)))?;
        write!(output, "Executing: ")?;
        output.reset()?;
        writeln!(output, "{cmd}")?;
        Ok(())
    }

    pub(super) fn print_updated(&self, patchname: &PatchName, applied: &[PatchName]) -> Result<()> {
        let mut output = self.output.borrow_mut();
        let (is_applied, is_top) = if let Some(pos) = applied.iter().position(|pn| pn == patchname)
        {
            (true, pos + 1 == applied.len())
        } else {
            (false, false)
        };
        let mut color_spec = termcolor::ColorSpec::new();
        output.set_color(color_spec.set_fg(Some(termcolor::Color::Cyan)))?;
        write!(output, "& ")?;
        color_spec.clear();
        output.set_color(
            color_spec
                .set_bold(is_top)
                .set_intense(is_applied && !is_top)
                .set_dimmed(!is_applied),
        )?;
        writeln!(output, "{patchname}")?;
        output.reset()?;
        Ok(())
    }
}
