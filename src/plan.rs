use std::path::PathBuf;

use crate::display;
use crate::runtime::{NodeCheck, PythonCheck};
use crate::syspkg::{PkgCheck, Pm};
use crate::yaml::SmallYaml;

pub struct Plan {
    pub steps: Vec<Step>,
    pub needs_sudo: bool,
}

pub struct Step {
    pub label: String,
    pub kind: StepKind,
}

pub enum StepKind {
    DownloadRuntime {
        name: String,
        version: String,
        url: String,
        sha256: String,
        dest: PathBuf,
    },
    InstallSystemPkgs {
        pm: String,
        packages: Vec<String>,
    },
    CreateVenv {
        python_path: PathBuf,
        venv_path: PathBuf,
    },
    PipInstall {
        venv_path: PathBuf,
        packages: Vec<String>,
        requirements_file: Option<String>,
    },
    NpmInstall,
    RunCommand {
        command: String,
        env_paths: Vec<PathBuf>,
    },
}

pub fn build(
    config: &SmallYaml,
    python_check: &Option<PythonCheck>,
    node_check: &Option<NodeCheck>,
    pm: &Option<Pm>,
    pkg_checks: &[PkgCheck],
) -> Plan {
    let mut steps = Vec::new();
    let mut needs_sudo = false;

    let mut env_paths: Vec<PathBuf> = Vec::new();
    let mut python_bin = PathBuf::from("python3");

    if let Some(check) = python_check {
        if let Some(sys) = &check.system {
            python_bin = sys.path.clone();
        }
        if let Some(dl) = &check.download {
            python_bin = dl.dest.join("bin").join("python3");
            env_paths.push(dl.dest.join("bin"));
            steps.push(Step {
                label: format!("Download Python {}", dl.version),
                kind: StepKind::DownloadRuntime {
                    name: dl.name.clone(),
                    version: dl.version.clone(),
                    url: dl.url.clone(),
                    sha256: dl.sha256.clone(),
                    dest: dl.dest.clone(),
                },
            });
        }
    }

    if let Some(check) = node_check
        && let Some(dl) = &check.download
    {
        env_paths.push(dl.dest.join("bin"));
        steps.push(Step {
            label: format!("Download Node {}", dl.version),
            kind: StepKind::DownloadRuntime {
                name: dl.name.clone(),
                version: dl.version.clone(),
                url: dl.url.clone(),
                sha256: dl.sha256.clone(),
                dest: dl.dest.clone(),
            },
        });
    }

    let missing: Vec<&PkgCheck> = pkg_checks.iter().filter(|p| p.missing).collect();
    if !missing.is_empty() {
        if let Some(Pm::Apt) = pm {
            needs_sudo = true;
        }
        let pm_name = match pm {
            Some(Pm::Brew) => "brew",
            Some(Pm::Apt) => "apt",
            None => "unknown",
        };
        let pkg_names: Vec<String> = missing.iter().map(|p| p.resolved.clone()).collect();
        steps.push(Step {
            label: format!("Install system packages: {}", pkg_names.join(", ")),
            kind: StepKind::InstallSystemPkgs {
                pm: pm_name.to_string(),
                packages: pkg_names,
            },
        });
    }

    for cmd in &config.pre_install {
        steps.push(Step {
            label: format!("pre_install: {cmd}"),
            kind: StepKind::RunCommand {
                command: cmd.clone(),
                env_paths: env_paths.clone(),
            },
        });
    }

    if config.project.language == "python" {
        let venv_path = PathBuf::from(".small_venv");
        steps.push(Step {
            label: "Create virtual environment".to_string(),
            kind: StepKind::CreateVenv {
                python_path: python_bin.clone(),
                venv_path: venv_path.clone(),
            },
        });
        env_paths.push(venv_path.join("bin"));

        let has_packages = !config.dependencies.python.is_empty();
        let has_req_file = config.requirements_file.is_some();
        if has_packages || has_req_file {
            let label = if has_req_file {
                "Install Python dependencies (pip install -r)".to_string()
            } else {
                "Install Python dependencies (pip install)".to_string()
            };
            steps.push(Step {
                label,
                kind: StepKind::PipInstall {
                    venv_path,
                    packages: config.dependencies.python.clone(),
                    requirements_file: config.requirements_file.clone(),
                },
            });
        }
    } else if config.project.language == "node" {
        steps.push(Step {
            label: "npm install".to_string(),
            kind: StepKind::NpmInstall,
        });
    }

    for cmd in &config.post_install {
        steps.push(Step {
            label: format!("post_install: {cmd}"),
            kind: StepKind::RunCommand {
                command: cmd.clone(),
                env_paths: env_paths.clone(),
            },
        });
    }

    if let Some(test_cmd) = &config.test {
        steps.push(Step {
            label: format!("Run tests: {test_cmd}"),
            kind: StepKind::RunCommand {
                command: test_cmd.clone(),
                env_paths,
            },
        });
    }

    Plan { steps, needs_sudo }
}

pub fn display(plan: &Plan) {
    if plan.steps.is_empty() {
        display::info("Nothing to install.");
        return;
    }
    println!();
    display::info("Installation plan:");
    for (i, step) in plan.steps.iter().enumerate() {
        println!("  {}. {}", i + 1, step.label);
    }
    if plan.needs_sudo {
        println!();
        display::info("Note: system package installation requires sudo");
    }
    println!();
}
