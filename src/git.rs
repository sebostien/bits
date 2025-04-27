use anyhow::Result;
use std::process::Command;

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

    pub fn rev_parse(text: &str) -> Option<String> {
        let output = Command::new("git")
            .args(["rev-parse", text])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Some(stdout.trim().to_string())
    }

    pub fn get_branch() -> Option<String> {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        (!stdout.is_empty()).then_some(stdout)
    }
}
