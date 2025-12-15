//! Script Runner Module
//!
//! This module provides functionality to detect and run project scripts
//! from various build systems (npm, cargo, make, etc.)
//!
//! Author: Huzeyfe Coşkun <huzeyfecoskun@hotmail.com>

use std::fs;
use std::path::{Path, PathBuf};

/// Represents a runnable script from a project
#[derive(Debug, Clone)]
pub struct ProjectScript {
    /// Display name of the script
    pub name: String,
    /// The command to run
    pub command: String,
    /// Optional description
    pub description: Option<String>,
    /// The project type this script belongs to
    pub project_type: ProjectType,
}

/// Supported project types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    /// Node.js project (package.json)
    Node,
    /// Rust project (Cargo.toml)
    Rust,
    /// Make project (Makefile)
    Make,
    /// Python project (pyproject.toml or setup.py)
    Python,
    /// Go project (go.mod)
    Go,
    /// Deno project (deno.json)
    Deno,
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::Node => write!(f, "npm"),
            ProjectType::Rust => write!(f, "cargo"),
            ProjectType::Make => write!(f, "make"),
            ProjectType::Python => write!(f, "python"),
            ProjectType::Go => write!(f, "go"),
            ProjectType::Deno => write!(f, "deno"),
        }
    }
}

/// Result of script detection
pub struct ScriptDetectionResult {
    /// Path to the project root
    pub project_root: PathBuf,
    /// Detected project type
    pub project_type: ProjectType,
    /// Available scripts
    pub scripts: Vec<ProjectScript>,
}

/// Detect project type and available scripts from a directory
/// This function collects ALL scripts from ALL detected project types
pub fn detect_scripts(dir: &Path) -> Option<ScriptDetectionResult> {
    log::info!("detect_scripts: scanning directory {:?}", dir);

    let mut all_scripts: Vec<ProjectScript> = Vec::new();
    let mut project_root: Option<PathBuf> = None;

    // Check each config file and collect scripts
    
    // 1. package.json (npm/node)
    let package_json = dir.join("package.json");
    if package_json.exists() {
        log::info!("detect_scripts: found package.json");
        if let Some(result) = detect_node_scripts(dir) {
            log::info!("detect_scripts: got {} npm scripts", result.scripts.len());
            if project_root.is_none() {
                project_root = Some(result.project_root.clone());
            }
            all_scripts.extend(result.scripts);
        }
    }

    // 2. Cargo.toml (rust)
    let cargo_toml = dir.join("Cargo.toml");
    if cargo_toml.exists() {
        log::info!("detect_scripts: found Cargo.toml");
        if let Some(result) = detect_cargo_scripts(dir) {
            log::info!("detect_scripts: got {} cargo scripts", result.scripts.len());
            if project_root.is_none() {
                project_root = Some(result.project_root.clone());
            }
            all_scripts.extend(result.scripts);
        } else {
            log::info!("detect_scripts: Cargo.toml has no [[bin]] or examples defined");
        }
    }

    // 3. Makefile
    let makefile = dir.join("Makefile");
    let makefile_lower = dir.join("makefile");
    let gnumakefile = dir.join("GNUmakefile");
    if makefile.exists() || makefile_lower.exists() || gnumakefile.exists() {
        log::info!("detect_scripts: found Makefile");
        if let Some(result) = detect_make_scripts(dir) {
            log::info!("detect_scripts: got {} make targets", result.scripts.len());
            if project_root.is_none() {
                project_root = Some(result.project_root.clone());
            }
            all_scripts.extend(result.scripts);
        }
    }

    // 4. deno.json
    let deno_json = dir.join("deno.json");
    let deno_jsonc = dir.join("deno.jsonc");
    if deno_json.exists() || deno_jsonc.exists() {
        log::info!("detect_scripts: found deno.json");
        if let Some(result) = detect_deno_scripts(dir) {
            log::info!("detect_scripts: got {} deno tasks", result.scripts.len());
            if project_root.is_none() {
                project_root = Some(result.project_root.clone());
            }
            all_scripts.extend(result.scripts);
        }
    }

    // 5. pyproject.toml (python)
    let pyproject = dir.join("pyproject.toml");
    if pyproject.exists() {
        log::info!("detect_scripts: found pyproject.toml");
        if let Some(result) = detect_python_scripts(dir) {
            log::info!("detect_scripts: got {} python scripts", result.scripts.len());
            if project_root.is_none() {
                project_root = Some(result.project_root.clone());
            }
            all_scripts.extend(result.scripts);
        }
    }

    log::info!("detect_scripts: total scripts found: {}", all_scripts.len());

    // Return combined result if any scripts found
    if all_scripts.is_empty() {
        log::info!("detect_scripts: no scripts found in any config file");
        return None;
    }

    // Sort scripts by project type for grouping
    all_scripts.sort_by(|a, b| {
        a.project_type
            .to_string()
            .cmp(&b.project_type.to_string())
            .then_with(|| a.name.cmp(&b.name))
    });

    Some(ScriptDetectionResult {
        project_root: project_root.unwrap_or_else(|| dir.to_path_buf()),
        project_type: ProjectType::Node, // Not used anymore
        scripts: all_scripts,
    })
}

/// Detect npm scripts from package.json
fn detect_node_scripts(dir: &Path) -> Option<ScriptDetectionResult> {
    let package_json = dir.join("package.json");
    if !package_json.exists() {
        log::info!("detect_node_scripts: package.json not found at {:?}", package_json);
        return None;
    }

    log::info!("detect_node_scripts: found package.json at {:?}", package_json);

    let content = fs::read_to_string(&package_json).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;

    let mut scripts = Vec::new();

    // Get scripts defined in package.json
    if let Some(scripts_obj) = json.get("scripts").and_then(|s| s.as_object()) {
        log::info!("detect_node_scripts: found {} scripts in package.json", scripts_obj.len());
        for (name, value) in scripts_obj {
            if let Some(cmd) = value.as_str() {
                log::info!("detect_node_scripts: adding script '{}' -> '{}'", name, cmd);
                scripts.push(ProjectScript {
                    name: name.clone(),
                    command: format!("npm run {}", name),
                    description: Some(cmd.to_string()),
                    project_type: ProjectType::Node,
                });
            }
        }
    } else {
        log::info!("detect_node_scripts: no scripts object found in package.json");
    }

    log::info!("detect_node_scripts: total scripts found: {}", scripts.len());

    // Only return if we found actual scripts
    if scripts.is_empty() {
        return None;
    }

    Some(ScriptDetectionResult {
        project_root: dir.to_path_buf(),
        project_type: ProjectType::Node,
        scripts,
    })
}

/// Detect cargo scripts from Cargo.toml
fn detect_cargo_scripts(dir: &Path) -> Option<ScriptDetectionResult> {
    let cargo_toml = dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        return None;
    }

    log::info!("detect_cargo_scripts: found Cargo.toml at {:?}", cargo_toml);

    let mut scripts = Vec::new();

    // Parse Cargo.toml for defined targets
    if let Ok(content) = fs::read_to_string(&cargo_toml) {
        if let Ok(toml) = content.parse::<toml::Value>() {
            // Check for binary targets defined in Cargo.toml
            if let Some(bin) = toml.get("bin").and_then(|b| b.as_array()) {
                for target in bin {
                    if let Some(name) = target.get("name").and_then(|n| n.as_str()) {
                        scripts.push(ProjectScript {
                            name: format!("run --bin {}", name),
                            command: format!("cargo run --bin {}", name),
                            description: Some(format!("Run {} binary", name)),
                            project_type: ProjectType::Rust,
                        });
                    }
                }
            }
        }
    }

    // Check for examples
    let examples_dir = dir.join("examples");
    if examples_dir.exists() {
        if let Ok(entries) = fs::read_dir(&examples_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "rs") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        scripts.push(ProjectScript {
                            name: format!("run --example {}", name),
                            command: format!("cargo run --example {}", name),
                            description: Some(format!("Run {} example", name)),
                            project_type: ProjectType::Rust,
                        });
                    }
                }
            }
        }
    }

    log::info!("detect_cargo_scripts: found {} cargo commands", scripts.len());

    if scripts.is_empty() {
        return None;
    }

    Some(ScriptDetectionResult {
        project_root: dir.to_path_buf(),
        project_type: ProjectType::Rust,
        scripts,
    })
}

/// Detect make targets from Makefile
fn detect_make_scripts(dir: &Path) -> Option<ScriptDetectionResult> {
    // Check for various Makefile names
    let makefile_names = ["Makefile", "makefile", "GNUmakefile"];
    let makefile = makefile_names
        .iter()
        .map(|name| dir.join(name))
        .find(|path| path.exists())?;

    let content = fs::read_to_string(&makefile).ok()?;
    let mut scripts = Vec::new();
    let mut seen_targets = std::collections::HashSet::new();

    // Parse Makefile targets (simple parsing)
    for line in content.lines() {
        // Skip comments and empty lines
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }

        // Look for target definitions (target: dependencies)
        if let Some(colon_pos) = line.find(':') {
            let target = line[..colon_pos].trim();

            // Skip pattern rules, special targets, and variable assignments
            if target.contains('%')
                || target.starts_with('.')
                || target.contains('$')
                || target.contains('=')
                || target.is_empty()
            {
                continue;
            }

            // Handle multiple targets on one line
            for t in target.split_whitespace() {
                if !seen_targets.contains(t) && !t.is_empty() {
                    seen_targets.insert(t.to_string());
                    scripts.push(ProjectScript {
                        name: t.to_string(),
                        command: format!("make {}", t),
                        description: None,
                        project_type: ProjectType::Make,
                    });
                }
            }
        }
    }

    // Only return if we found actual targets
    if scripts.is_empty() {
        return None;
    }

    Some(ScriptDetectionResult {
        project_root: dir.to_path_buf(),
        project_type: ProjectType::Make,
        scripts,
    })
}

/// Detect deno tasks from deno.json
fn detect_deno_scripts(dir: &Path) -> Option<ScriptDetectionResult> {
    let deno_json = dir.join("deno.json");
    let deno_jsonc = dir.join("deno.jsonc");

    let config_path = if deno_json.exists() {
        deno_json
    } else if deno_jsonc.exists() {
        deno_jsonc
    } else {
        return None;
    };

    let content = fs::read_to_string(&config_path).ok()?;
    // Remove comments for jsonc
    let content = content
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    let json: serde_json::Value = serde_json::from_str(&content).ok()?;

    let mut scripts = Vec::new();

    // Only get tasks defined in deno.json
    if let Some(tasks) = json.get("tasks").and_then(|t| t.as_object()) {
        for (name, value) in tasks {
            if let Some(cmd) = value.as_str() {
                scripts.push(ProjectScript {
                    name: name.clone(),
                    command: format!("deno task {}", name),
                    description: Some(cmd.to_string()),
                    project_type: ProjectType::Deno,
                });
            }
        }
    }

    if scripts.is_empty() {
        return None;
    }

    Some(ScriptDetectionResult {
        project_root: dir.to_path_buf(),
        project_type: ProjectType::Deno,
        scripts,
    })
}

/// Detect python scripts from pyproject.toml
fn detect_python_scripts(dir: &Path) -> Option<ScriptDetectionResult> {
    let pyproject = dir.join("pyproject.toml");

    if !pyproject.exists() {
        return None;
    }

    let mut scripts = Vec::new();

    // Parse pyproject.toml for defined scripts only
    if let Ok(content) = fs::read_to_string(&pyproject) {
        if let Ok(toml) = content.parse::<toml::Value>() {
            // Check for poetry scripts
            if let Some(poetry_scripts) = toml
                .get("tool")
                .and_then(|t| t.get("poetry"))
                .and_then(|p| p.get("scripts"))
                .and_then(|s| s.as_table())
            {
                for (name, _) in poetry_scripts {
                    scripts.push(ProjectScript {
                        name: format!("poetry run {}", name),
                        command: format!("poetry run {}", name),
                        description: None,
                        project_type: ProjectType::Python,
                    });
                }
            }

            // Check for PDM scripts
            if let Some(pdm_scripts) = toml
                .get("tool")
                .and_then(|t| t.get("pdm"))
                .and_then(|p| p.get("scripts"))
                .and_then(|s| s.as_table())
            {
                for (name, _) in pdm_scripts {
                    scripts.push(ProjectScript {
                        name: format!("pdm run {}", name),
                        command: format!("pdm run {}", name),
                        description: None,
                        project_type: ProjectType::Python,
                    });
                }
            }
        }
    }

    if scripts.is_empty() {
        return None;
    }

    Some(ScriptDetectionResult {
        project_root: dir.to_path_buf(),
        project_type: ProjectType::Python,
        scripts,
    })
}

/// Detect go commands - Go doesn't have a script definition concept
/// Users should use Makefile or Taskfile for Go projects
fn detect_go_scripts(_dir: &Path) -> Option<ScriptDetectionResult> {
    // Go doesn't have a native script definition concept like package.json
    // go.mod only defines module and dependencies, not scripts
    // For Go projects, use Makefile which is already supported
    None
}

/// Find project root by walking up the directory tree
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut current = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };

    loop {
        // Check for project markers
        let markers = [
            "package.json",
            "Cargo.toml",
            "Makefile",
            "makefile",
            "GNUmakefile",
            "go.mod",
            "deno.json",
            "deno.jsonc",
            "pyproject.toml",
            "setup.py",
            ".git",
        ];

        for marker in markers {
            if current.join(marker).exists() {
                return Some(current);
            }
        }

        if !current.pop() {
            break;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_node_scripts() {
        let dir = tempdir().unwrap();
        let package_json = r#"{
            "name": "test",
            "scripts": {
                "build": "tsc",
                "test": "jest",
                "start": "node index.js"
            }
        }"#;
        fs::write(dir.path().join("package.json"), package_json).unwrap();

        let result = detect_scripts(dir.path()).unwrap();
        assert_eq!(result.project_type, ProjectType::Node);
        assert!(result.scripts.iter().any(|s| s.name == "build"));
        assert!(result.scripts.iter().any(|s| s.name == "test"));
        assert!(result.scripts.iter().any(|s| s.name == "start"));
    }

    #[test]
    fn test_detect_cargo_scripts() {
        let dir = tempdir().unwrap();
        let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
"#;
        fs::write(dir.path().join("Cargo.toml"), cargo_toml).unwrap();

        let result = detect_scripts(dir.path()).unwrap();
        assert_eq!(result.project_type, ProjectType::Rust);
        assert!(result.scripts.iter().any(|s| s.name == "build"));
        assert!(result.scripts.iter().any(|s| s.name == "run"));
        assert!(result.scripts.iter().any(|s| s.name == "test"));
    }

    #[test]
    fn test_find_project_root() {
        let dir = tempdir().unwrap();
        let sub_dir = dir.path().join("src").join("lib");
        fs::create_dir_all(&sub_dir).unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();

        let root = find_project_root(&sub_dir).unwrap();
        assert_eq!(root, dir.path());
    }
}
