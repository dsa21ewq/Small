use std::path::PathBuf;

use crate::display;
use crate::runtime;
use crate::yaml;

pub fn run() -> anyhow::Result<()> {
    let config = yaml::parse("small.yaml")?;

    let mut env_paths = Vec::new();

    if config.project.language == "python" {
        if let Some(constraint) = &config.runtimes.python {
            let check = runtime::check_python(constraint)?;
            if let Some(sys) = check.system {
                env_paths.push(sys.path.parent().unwrap_or(&sys.path).to_path_buf());
            } else if let Some(dl) = check.download {
                env_paths.push(dl.dest.join("bin"));
            }
        }
        env_paths.push(PathBuf::from(".small_venv/bin"));
    } else if config.project.language == "node"
        && let Some(constraint) = &config.runtimes.node
    {
        let check = runtime::check_node(constraint)?;
        if let Some(sys) = check.system {
            env_paths.push(sys.path.parent().unwrap_or(&sys.path).to_path_buf());
        } else if let Some(dl) = check.download {
            env_paths.push(dl.dest.join("bin"));
        }
    }

    let current_path = std::env::var("PATH").unwrap_or_default();
    let mut cmd = duct::cmd("sh", ["-c", &config.entrypoint]);
    if !env_paths.is_empty() {
        let extra: Vec<String> = env_paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        let new_path = extra.join(":") + ":" + &current_path;
        cmd = cmd.env("PATH", new_path);
    }

    display::step(&format!("Running: {}", config.entrypoint));
    cmd.run()?;
    Ok(())
}
