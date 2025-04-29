use anyhow::{anyhow, Result};
use log::error;
use prettytable::{format, Cell, Row, Table};
use std::{collections::HashMap, process::Command};

pub struct Git {}

impl Git {
    pub fn get_remote() -> Result<Option<String>> {
        let output = Command::new("git").args(["remote", "-v"]).output()?;

        if !output.status.success() {
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.trim().lines() {
            if let Some((_, rest)) = line.trim().split_once('\t') {
                if let Some((remote, ty)) = rest.trim().split_once(' ') {
                    if ty == "(push)" {
                        return Ok(Some(remote.to_string()));
                    }
                }
            }
        }

        Ok(None)
    }

    pub fn rev_parse(text: &str) -> Result<Option<String>> {
        let output = Command::new("git").args(["rev-parse", text]).output()?;

        if !output.status.success() {
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(Some(stdout.trim().to_string()))
    }

    pub fn get_branch() -> Result<Option<String>> {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .output()?;

        if !output.status.success() {
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok((!stdout.is_empty()).then_some(stdout))
    }

    pub fn branches(authors: &[String]) -> Result<()> {
        let outputs = [
            "%(authorname)",
            "%(authordate:iso8601)",
            "%(refname)",
            "%(contents:subject)",
        ]
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

        let mut matches = HashMap::new();

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

            let mut is_local = this_ref.is_local;
            let mut is_remote = this_ref.is_remote;

            if !authors_filter.is_empty()
                && !authors_filter
                    .iter()
                    .any(|f| f.iter().all(|f| author_vec.contains(&f.as_str())))
            {
                is_local = false;
                is_remote = false;
                this_ref.is_local = false;
                this_ref.is_remote = false;
            }

            let entry = matches.entry(this_ref.refname).or_insert(this_ref);
            entry.is_local = entry.is_local || is_local;
            entry.is_remote = entry.is_remote || is_remote;
        }

        let mut matches = matches.into_values().collect::<Vec<_>>();
        matches.sort_unstable();

        let mut table = Table::new();
        table.set_titles(Row::new(vec![
            Cell::new("Type"),
            Cell::new("Name"),
            Cell::new("Author"),
            Cell::new("Date"),
            Cell::new("Subject"),
        ]));

        table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

        for m in matches {
            if let Some(row) = m.into() {
                table.add_row(row);
            }
        }

        println!("{table}");

        Ok(())
    }
}

#[derive(Debug, Eq, Ord)]
struct ForEachRef<'a> {
    author_name: &'a str,
    author_date: &'a str,
    refname: &'a str,
    subject: &'a str,
    is_remote: bool,
    is_local: bool,
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
        self.author_name == other.author_name && self.refname == other.refname
    }
}

impl From<ForEachRef<'_>> for Option<Row> {
    fn from(value: ForEachRef<'_>) -> Self {
        let ty = match (value.is_local, value.is_remote) {
            (true, true) => "Both",
            (false, true) => "Remote",
            (true, false) => "Local",
            (false, false) => {
                return None;
            }
        };

        Some(Row::new(vec![
            Cell::new(ty),
            Cell::new(value.refname),
            Cell::new(value.author_name),
            Cell::new(value.author_date),
            Cell::new(value.subject),
        ]))
    }
}

impl<'a> ForEachRef<'a> {
    fn from_output(output: &'a str) -> Result<Option<Self>, String> {
        let line = output.trim().split('\0').collect::<Vec<_>>();

        if line.len() != 4 {
            return Err(format!(
                "Unexpected result returned trying to parse for-each-ref: '{output}'"
            ));
        }

        let author_name = line[0];
        let author_date = line[1];
        let mut refname = line[2];
        let subject = line[3];

        let mut is_remote = false;
        let mut is_local = false;

        if refname.starts_with("refs/tags") {
            return Ok(None);
        }

        if refname.starts_with("refs/heads/") {
            refname = &refname["refs/heads/".len()..];
            is_local = true;
        } else if refname.starts_with("refs/remotes/origin/") {
            refname = &refname["refs/remotes/origin/".len()..];
            is_remote = true;
        }

        Ok(Some(Self {
            author_name,
            author_date,
            refname,
            subject,
            is_remote,
            is_local,
        }))
    }
}

fn split_authors(authors: &[String]) -> Vec<Vec<String>> {
    authors
        .into_iter()
        .map(|a| a.split_whitespace().map(|a| a.to_lowercase()).collect())
        .collect()
}
