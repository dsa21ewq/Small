use std::io::IsTerminal;

use crate::display;
use crate::plan;
use crate::runtime;
use crate::syspkg;
use crate::yaml;

pub fn run(skip_confirm: bool) -> anyhow::Result<()> {
    display::step("Parsing small.yaml...");
    let config = yaml::parse("small.yaml")?;
    display::success("Parsed small.yaml");

    let os = runtime::detect_os();
    display::info(&format!("OS: {} ({})", os.os, os.arch));

    let python_check;
    let node_check;
    if config.project.language == "python" {
        let constraint = config.runtimes.python.as_deref().unwrap_or(">=3.9");
        python_check = Some(runtime::check_python(constraint)?);
        node_check = None;
        let pc = python_check.as_ref().unwrap();
        if let Some(sys) = &pc.system {
            display::info(&format!(
                "Python {} found at {}",
                sys.version,
                sys.path.display()
            ));
        } else if let Some(dl) = &pc.download {
            display::info(&format!("Python {} will be downloaded", dl.version));
        } else {
            anyhow::bail!("No Python runtime satisfies constraint: {constraint}");
        }
    } else {
        let constraint = config.runtimes.node.as_deref().unwrap_or(">=18");
        node_check = Some(runtime::check_node(constraint)?);
        python_check = None;
        let nc = node_check.as_ref().unwrap();
        if let Some(sys) = &nc.system {
            display::info(&format!(
                "Node {} found at {}",
                sys.version,
                sys.path.display()
            ));
        } else if let Some(dl) = &nc.download {
            display::info(&format!("Node {} will be downloaded", dl.version));
        } else {
            anyhow::bail!("No Node runtime satisfies constraint: {constraint}");
        }
    }

    let pm = syspkg::detect_pm();
    let pkg_checks = if !config.system.is_empty() {
        if let Some(ref pm) = pm {
            display::info(&format!("Package manager: {:?}", pm));
            syspkg::check_packages(&config.system, pm)
        } else {
            display::info("No supported package manager found, skipping system packages");
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let plan = plan::build(&config, &python_check, &node_check, &pm, &pkg_checks);
    plan::display(&plan);

    if plan.steps.is_empty() {
        display::success("Nothing to install.");
        return Ok(());
    }

    if skip_confirm || !std::io::stdin().is_terminal() {
        display::info("Skipping confirmation (non-interactive mode)");
    } else if !confirm_install(&plan)? {
        display::info("Install cancelled.");
        return Ok(());
    }

    let cwd = std::env::current_dir()?;
    if let Err(e) = crate::executor::execute(&plan, &cwd) {
        display::error(&format!("Install failed: {e}"));
        crate::clean::remove_artifacts();
        display::info("Run 'small install' again to retry from scratch.");
        std::process::exit(1);
    }

    display::success("Install complete. Run 'small run' to start.");
    Ok(())
}

fn confirm_install(plan: &plan::Plan) -> anyhow::Result<bool> {
    use dialoguer::Confirm;
    let prompt = if plan.needs_sudo {
        "Execute plan (requires sudo)?"
    } else {
        "Execute plan?"
    };
    Confirm::new()
        .with_prompt(prompt)
        .default(true)
        .interact()
        .map_err(Into::into)
}
