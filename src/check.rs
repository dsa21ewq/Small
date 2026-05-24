use crate::display;
use crate::plan;
use crate::runtime;
use crate::syspkg;
use crate::yaml;

pub fn run() -> anyhow::Result<()> {
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
            display::info(&format!(
                "Warning: No Python runtime satisfies {constraint}"
            ));
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
            display::info(&format!("Warning: No Node runtime satisfies {constraint}"));
        }
    }

    let pm = syspkg::detect_pm();
    let pkg_checks = if !config.system.is_empty() {
        if let Some(ref pm) = pm {
            syspkg::check_packages(&config.system, pm)
        } else {
            display::info("No supported package manager found");
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let plan = plan::build(&config, &python_check, &node_check, &pm, &pkg_checks);
    plan::display(&plan);

    display::info("Dry-run complete. Run 'small install' to execute.");
    Ok(())
}
