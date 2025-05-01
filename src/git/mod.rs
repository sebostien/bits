use anyhow::Result;
use std::process::Command;

mod git_ref;

pub use git_ref::GitRefField;

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
}
