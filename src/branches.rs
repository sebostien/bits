use anyhow::{anyhow, Result};
use log::error;
use prettytable::color::{BLUE, BRIGHT_BLACK, GREEN};
use prettytable::format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR;
use prettytable::{color::RED, Attr, Cell, Row, Table};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

use crate::git::GitRefField;

#[derive(Serialize, Deserialize)]
pub struct Branches {}

impl Default for Branches {
    fn default() -> Self {
        Self {}
    }
}

impl Branches {
    pub fn list(authors: &[String], include_remotes: bool) -> Result<()> {
        let outputs = [
            GitRefField::AuthorName,
            GitRefField::AuthorDateISO,
            GitRefField::RefName,
            GitRefField::ObjectName,
            GitRefField::Subject,
        ]
        .map(<&str>::from)
        .join("%00");

        let output = Command::new("git")
            .args([
                "for-each-ref",
                &format!("--format={outputs}"),
                "--sort=authordate",
            ])
            .output()?;

        if !output.status.success() {
            error!("{}", String::from_utf8_lossy(&output.stderr));
            return Err(anyhow!("Could not detect branches"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let authors_filter = split_authors(authors);

        let mut matches: HashMap<&str, ForEachRef<'_>> = HashMap::new();

        for line in stdout.trim().lines() {
            let mut this_ref = match ForEachRef::from_output(line) {
                Ok(Some(x)) => x,
                Ok(None) => {
                    continue;
                }
                Err(e) => {
                    error!("{e}");
                    continue;
                }
            };

            let author_vec = this_ref.author_name.to_lowercase();
            let author_vec = author_vec.split_whitespace().collect::<Vec<_>>();

            if !authors_filter.is_empty()
                && !authors_filter
                    .iter()
                    .any(|f| f.iter().all(|f| author_vec.contains(&f.as_str())))
            {
                this_ref.is_local = false;
                this_ref.is_remote = false;
            }

            if let Some(prev) = matches.get_mut(this_ref.ref_name) {
                prev.is_local = prev.is_local || this_ref.is_local;
                prev.is_remote = prev.is_remote || this_ref.is_remote;

                if prev.object_name != this_ref.object_name {
                    prev.diverged = true;
                    if prev.is_remote {
                        // The other is remote. Let's keep the local info
                        prev.subject = this_ref.subject;
                        prev.object_name = this_ref.object_name;
                        prev.author_name = this_ref.author_name;
                        prev.author_date = this_ref.author_date;
                    }
                }
            } else {
                matches.insert(this_ref.ref_name, this_ref);
            }
        }

        let mut matches = matches.into_values().collect::<Vec<_>>();
        matches.sort_unstable();

        let mut table = Table::new();
        table.set_titles(Row::new(vec![
            Cell::new("Type"),
            Cell::new("Name"),
            Cell::new("Author"),
            Cell::new("Updated"),
            Cell::new("Subject"),
        ]));

        table.set_format(*FORMAT_NO_BORDER_LINE_SEPARATOR);

        for m in matches {
            if let Some(row) = m.to_row(include_remotes) {
                table.add_row(row);
            }
        }

        table.print_tty(false)?;

        Ok(())
    }
}

#[derive(Debug, Eq, Ord)]
struct ForEachRef<'a> {
    author_name: &'a str,
    author_date: &'a str,
    ref_name: &'a str,
    object_name: &'a str,
    subject: &'a str,
    is_remote: bool,
    is_local: bool,
    diverged: bool,
}

impl<'a> PartialOrd for ForEachRef<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.is_local.partial_cmp(&other.is_local) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.is_remote.partial_cmp(&other.is_remote) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.author_date.partial_cmp(other.author_date) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.author_name.partial_cmp(other.author_name)
    }
}

impl<'a> PartialEq for ForEachRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.author_name == other.author_name && self.ref_name == other.ref_name
    }
}

impl<'a> ForEachRef<'a> {
    fn from_output(output: &'a str) -> Result<Option<Self>, String> {
        let line = output.trim().split('\0').collect::<Vec<_>>();

        if line.len() != 5 {
            return Err(format!(
                "Unexpected result returned trying to parse for-each-ref: '{output}'"
            ));
        }

        let author_name = line[0];
        let author_date = line[1];
        let mut ref_name = line[2];
        let object_name = line[3];
        let subject = line[4];

        let mut is_remote = false;
        let mut is_local = false;

        if ref_name.starts_with("refs/tags") {
            return Ok(None);
        }

        if ref_name.starts_with("refs/heads/") {
            ref_name = &ref_name["refs/heads/".len()..];
            is_local = true;
        } else if ref_name.starts_with("refs/remotes/origin/") {
            ref_name = &ref_name["refs/remotes/origin/".len()..];
            is_remote = true;
        }

        Ok(Some(Self {
            author_name,
            author_date,
            ref_name,
            object_name,
            subject,
            is_remote,
            is_local,
            diverged: false,
        }))
    }

    fn to_row(self, include_remotes: bool) -> Option<Row> {
        if self.diverged {
            assert!(self.is_local);
            assert!(self.is_remote);
        }

        let ty = match (self.is_local, self.is_remote) {
            (true, true) if self.diverged => Cell::new("D")
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(RED)),
            (true, true) => Cell::new("B")
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(GREEN)),
            (true, false) => Cell::new("L").with_style(Attr::ForegroundColor(BLUE)),
            (false, true) if include_remotes => Cell::new("R")
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(BRIGHT_BLACK)),
            _ => {
                return None;
            }
        };

        Some(Row::new(vec![
            ty,
            Cell::new(self.ref_name),
            Cell::new(self.author_name),
            Cell::new(self.author_date),
            Cell::new(self.subject),
        ]))
    }
}

fn split_authors(authors: &[String]) -> Vec<Vec<String>> {
    authors
        .into_iter()
        .map(|a| a.split_whitespace().map(|a| a.to_lowercase()).collect())
        .collect()
}
