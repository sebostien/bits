use anyhow::{anyhow, Result};
use log::{error, info};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::git::Git;

#[derive(Serialize, Deserialize)]
pub struct Open {
    program: PathBuf,
    patterns: Vec<PatternOpen>,
    git: HashMap<String, GitOpen>,
}

impl Default for Open {
    fn default() -> Self {
        Self {
            program: "xdg-open".into(),
            patterns: Vec::new(),
            git: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PatternOpen {
    #[serde(default)]
    priority: i32,
    pattern: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitOpen {
    #[serde(default)]
    priority: i32,
    remote: String,
    url: String,
    branch: Option<String>,
    commit: Option<String>,
    patterns: Vec<PatternOpen>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitPrOpen {
    pattern: String,
    url: String,
}

#[derive(Debug, PartialEq, Eq)]
struct CanOpen {
    priority: i32,
    url: String,
}

impl Open {
    pub fn open(&self, text: &str) -> Result<()> {
        let mut can = Vec::new();

        can.append(&mut self.open_git(text));
        can.append(&mut self.open_pattern(text));

        can.sort_by_key(|CanOpen { priority, .. }| *priority);

        if let Some(first) = can.pop() {
            self.open_str(&first.url)
        } else {
            Err(anyhow!("Could not find pattern for {text}"))
        }
    }

    fn open_pattern(&self, text: &str) -> Vec<CanOpen> {
        let mut can = Vec::new();
        for pattern in &self.patterns {
            if let Some(url) = pattern.get_match(text) {
                can.push(CanOpen {
                    priority: pattern.priority,
                    url,
                });
            }
        }

        can
    }

    fn open_git(&self, text: &str) -> Vec<CanOpen> {
        let mut can = Vec::new();

        let (is_commit, text) = if text == "." {
            (false, text.to_string())
        } else if let Some(real_commit) = Git::rev_parse(text).ok().flatten() {
            (true, real_commit)
        } else {
            (false, text.to_string())
        };

        match Git::get_remote() {
            Ok(Some(remote)) => {
                for git_open in self.git.values() {
                    if let Some(mut url) = git_open.get_base(&text, is_commit) {
                        let regex = Regex::new(&git_open.remote).unwrap();
                        let groups = match regex.captures(&remote) {
                            None => continue,
                            Some(g) => g,
                        };

                        for (i, group) in groups.iter().skip(1).enumerate() {
                            url =
                                url.replacen(&format!("<r{}>", i + 1), group.unwrap().as_str(), 1);
                        }

                        can.push(CanOpen {
                            priority: git_open.priority,
                            url,
                        });
                    }
                }
            }
            Ok(None) => {}
            Err(e) => {
                error!("{e}");
            }
        }

        can
    }

    fn open_str(&self, s: &str) -> Result<()> {
        info!("Opening: {s}");

        let prog_canon = self.program.canonicalize();
        let program = if let Ok(p) = prog_canon.as_ref() {
            p
        } else {
            &self.program
        };

        Command::new(program)
            .args([s])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        Ok(())
    }
}

impl GitOpen {
    fn get_base(&self, text: &str, is_commit: bool) -> Option<String> {
        if text == "." {
            if let Some(branch_url) = &self.branch {
                if let Some(branch) = Git::get_branch().ok().flatten() {
                    return Some(branch_url.replacen("<branch>", &branch, 1));
                }
            }

            return Some(self.url.clone());
        }

        if is_commit {
            if let Some(commit) = &self.commit {
                return Some(commit.replacen("<commit>", text, 1));
            }

            return None;
        }

        for pattern in &self.patterns {
            if let Some(pat) = pattern.get_match(text) {
                return Some(pat);
            }
        }

        None
    }
}

impl PatternOpen {
    fn get_match(&self, text: &str) -> Option<String> {
        let pattern = Regex::new(&self.pattern).unwrap();

        let groups = pattern.captures(text)?;
        let mut url = self.url.clone();
        for (i, group) in groups.iter().skip(1).enumerate() {
            url = url.replacen(&format!("<pat{}>", i + 1), group.unwrap().as_str(), 1);
        }

        Some(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn get_open_config() -> Open {
        Open {
            program: "echo".into(),
            patterns: vec![PatternOpen {
                priority: 1,
                pattern: r"test-(\d+)".to_string(),
                url: "https://example.com/<pat1>".to_string(),
            }],
            git: HashMap::new(),
        }
    }

    #[test]
    fn test_open_pattern() {
        let open = get_open_config();
        let can = open.open_pattern("test-123");

        assert_eq!(can.len(), 1);
        assert_eq!(can[0].url, "https://example.com/123");
        assert_eq!(can[0].priority, 1);

        let can = open.open_pattern("feature-123");
        assert!(can.is_empty());
    }

    fn get_git_open_config() -> GitOpen {
        GitOpen {
            priority: 1,
            remote: "https?://repo.com/(\\.*).git".to_string(),
            url: "https://repo.com/<r1>/".to_string(),
            branch: None,
            commit: Some("https://repo.com/<r1>/<commit>".to_string()),
            patterns: vec![PatternOpen {
                priority: 2,
                pattern: "^(\\d+)$".to_string(),
                url: "https://repo.com/<r1>/p1/<pat1>".to_string(),
            }],
        }
    }

    #[test]
    fn test_git_open_get_base_url() {
        let git_open = get_git_open_config();

        assert_eq!(
            git_open.get_base(".", false),
            Some("https://repo.com/<r1>/".to_string())
        );
    }

    #[test]
    fn test_git_get_base_commit() {
        let git_open = get_git_open_config();

        assert_eq!(
            git_open.get_base("abc", true),
            Some("https://repo.com/<r1>/abc".to_string())
        );
        assert_eq!(
            git_open.get_base("helloa", true),
            Some("https://repo.com/<r1>/helloa".to_string())
        );
        assert_eq!(
            git_open.get_base("", true),
            Some("https://repo.com/<r1>/".to_string())
        );
        assert_eq!(
            git_open.get_base("1", true),
            Some("https://repo.com/<r1>/1".to_string())
        );
        assert_eq!(git_open.get_base("", false), None,);
        assert_eq!(git_open.get_base("abc", false), None,);
    }

    #[test]
    fn test_git_get_base_pattern() {
        let git_open = get_git_open_config();

        assert_eq!(
            git_open.get_base("123", false),
            Some("https://repo.com/<r1>/p1/123".to_string())
        );
        assert_eq!(
            git_open.get_base("1", false),
            Some("https://repo.com/<r1>/p1/1".to_string())
        );
        assert_eq!(git_open.get_base("a", false), None);
        assert_eq!(git_open.get_base("abc", false), None,);
    }
}
