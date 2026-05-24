use std::path::Path;

use crate::display;
use crate::plan::{Plan, StepKind};
use crate::{runtime, syspkg};

pub fn execute(plan: &Plan, project_dir: &Path) -> anyhow::Result<()> {
    for step in &plan.steps {
        display::step(&step.label);
        execute_step(&step.kind, project_dir)?;
        display::success(&step.label);
    }
    Ok(())
}

fn execute_step(kind: &StepKind, project_dir: &Path) -> anyhow::Result<()> {
    match kind {
        StepKind::DownloadRuntime {
            name,
            version,
            url,
            sha256,
            dest,
        } => {
            let cache_file =
                dest.join("bin")
                    .join(if name == "python" { "python3" } else { "node" });
            if cache_file.exists() {
                display::info(&format!("{name} {version} already cached"));
                return Ok(());
            }
            runtime::download_and_extract(url, sha256, dest)
        }
        StepKind::InstallSystemPkgs { pm, packages } => {
            let pm_enum = match pm.as_str() {
                "brew" => syspkg::Pm::Brew,
                "apt" => syspkg::Pm::Apt,
                _ => anyhow::bail!("unknown package manager: {pm}"),
            };
            syspkg::install_packages(&pm_enum, packages)
        }
        StepKind::CreateVenv {
            python_path,
            venv_path,
        } => {
            let _ = std::fs::remove_dir_all(venv_path);
            let venv_str = venv_path.to_string_lossy().to_string();
            duct::cmd(python_path, ["-m", "venv", &venv_str])
                .dir(project_dir)
                .run()?;
            Ok(())
        }
        StepKind::PipInstall {
            venv_path,
            packages,
            requirements_file,
        } => {
            let pip = project_dir.join(venv_path).join("bin").join("pip");
            if let Some(req_file) = requirements_file {
                duct::cmd(&pip, ["install", "-r", req_file])
                    .dir(project_dir)
                    .run()?;
            }
            if !packages.is_empty() {
                let mut args = vec!["install"];
                args.extend(packages.iter().map(|s| s.as_str()));
                duct::cmd(&pip, args).dir(project_dir).run()?;
            }
            Ok(())
        }
        StepKind::NpmInstall => {
            duct::cmd("npm", ["install"]).dir(project_dir).run()?;
            Ok(())
        }
        StepKind::RunCommand { command, env_paths } => {
            let mut cmd = duct::cmd("sh", ["-c", command]);
            cmd = cmd.dir(project_dir);
            if !env_paths.is_empty() {
                let current_path = std::env::var("PATH").unwrap_or_default();
                let extra: Vec<String> = env_paths
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();
                let new_path = extra.join(":") + ":" + &current_path;
                cmd = cmd.env("PATH", new_path);
            }
            cmd.run()?;
            Ok(())
        }
    }
}
