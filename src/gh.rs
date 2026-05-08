use std::path::Path;
use std::process::Command;

/// Result of a `gh` CLI invocation, ready to render in a panel.
pub struct GhOutput {
    pub title: String,
    pub body: String,
}

pub fn pr_status(repo_root: &Path, branch: Option<&str>) -> GhOutput {
    if !gh_available() {
        return GhOutput {
            title: "GitHub CLI".to_string(),
            body: "gh is not installed. See https://cli.github.com/".to_string(),
        };
    }

    let title = match branch {
        Some(b) => format!("gh pr — branch {}", b),
        None => "gh pr status".to_string(),
    };

    let mut cmd = Command::new("gh");
    cmd.current_dir(repo_root);
    if let Some(b) = branch {
        cmd.args(["pr", "view", b]);
    } else {
        cmd.args(["pr", "status"]);
    }

    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            return GhOutput {
                title,
                body: format!("Failed to invoke gh: {e}"),
            };
        }
    };

    let body = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).into_owned()
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        format!(
            "gh exited {}\n\n{}",
            output.status.code().unwrap_or(-1),
            stderr
        )
    };

    GhOutput { title, body }
}

fn gh_available() -> bool {
    Command::new("gh")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
