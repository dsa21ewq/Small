use std::fs;
use std::path::Path;

use dialoguer::{Confirm, Input};

use crate::yaml::{Project, Runtimes, SmallYaml};

struct ScanResult {
    language: String,
    has_requirements_txt: bool,
    has_package_json: bool,
    system_pkgs: Vec<String>,
    entrypoint: String,
    test_cmd: String,
}

pub fn run() -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dir_name = cwd
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "my-project".to_string());

    let scan = scan_dir(&cwd);

    let name: String = Input::new()
        .with_prompt("Project name")
        .default(dir_name)
        .interact_text()?;

    let language: String = Input::new()
        .with_prompt("Language")
        .default(scan.language)
        .interact_text()?;

    let version_prompt = if language == "python" {
        "Python version requirement"
    } else {
        "Node version requirement"
    };
    let version_default = if language == "python" {
        ">=3.9,<3.13"
    } else {
        ">=18"
    };
    let version: String = Input::new()
        .with_prompt(version_prompt)
        .default(version_default.to_string())
        .interact_text()?;

    let mut requirements_file = None;
    if scan.has_requirements_txt
        && Confirm::new()
            .with_prompt("Found requirements.txt. Use requirements.txt as dependency source?")
            .interact()?
    {
        requirements_file = Some("requirements.txt".to_string());
    }

    let mut package_file = None;
    if scan.has_package_json
        && language == "node"
        && Confirm::new()
            .with_prompt("Found package.json. Use package.json as dependency source?")
            .interact()?
    {
        package_file = Some("package.json".to_string());
    }

    let mut system_pkgs = Vec::new();
    for pkg in &scan.system_pkgs {
        let prompt = format!("Detected {pkg} indicator. Add {pkg} to system dependencies?");
        if Confirm::new().with_prompt(prompt).interact()? {
            system_pkgs.push(pkg.clone());
        }
    }

    let entrypoint: String = Input::new()
        .with_prompt("Entrypoint command")
        .default(scan.entrypoint)
        .interact_text()?;

    let test: String = Input::new()
        .with_prompt("Test command")
        .default(scan.test_cmd)
        .interact_text()?;

    let config = SmallYaml {
        project: Project {
            name,
            language: language.clone(),
        },
        runtimes: Runtimes {
            python: if language == "python" {
                Some(version.clone())
            } else {
                None
            },
            node: if language == "node" {
                Some(version)
            } else {
                None
            },
        },
        system: system_pkgs,
        dependencies: Default::default(),
        requirements_file,
        package_file,
        env: Default::default(),
        pre_install: vec![],
        post_install: vec![],
        test: if test.is_empty() { None } else { Some(test) },
        entrypoint,
    };

    let yaml_str = serde_yaml::to_string(&config)?;
    fs::write("small.yaml", yaml_str)?;
    println!("Generated small.yaml");

    Ok(())
}

fn scan_dir(dir: &Path) -> ScanResult {
    let mut scan = ScanResult {
        language: "python".to_string(),
        has_requirements_txt: false,
        has_package_json: false,
        system_pkgs: Vec::new(),
        entrypoint: String::new(),
        test_cmd: String::new(),
    };

    let entries: Vec<String> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    if entries.contains(&"package.json".to_string())
        && !entries.contains(&"requirements.txt".to_string())
        && !entries.contains(&"setup.py".to_string())
        && !entries.contains(&"pyproject.toml".to_string())
    {
        scan.language = "node".to_string();
    }

    if entries.contains(&"requirements.txt".to_string()) {
        scan.has_requirements_txt = true;
    }
    if entries.contains(&"package.json".to_string()) {
        scan.has_package_json = true;
    }

    if entries.contains(&"CMakeLists.txt".to_string()) {
        scan.system_pkgs.push("cmake".to_string());
    }

    let has_configure = entries
        .iter()
        .any(|e| e == "configure" || e == "configure.ac");
    if has_configure {
        scan.system_pkgs.push("pkg-config".to_string());
    }

    if scan.language == "python" {
        for candidate in &[
            "main.py",
            "app.py",
            "manage.py",
            "server.py",
            "run.py",
            "src/main.py",
        ] {
            if dir.join(candidate).exists() {
                scan.entrypoint = format!("python {candidate}");
                break;
            }
        }
        if scan.entrypoint.is_empty() {
            for name in &entries {
                if name.ends_with(".py") && name != "setup.py" && dir.join(name).is_file() {
                    scan.entrypoint = format!("python {name}");
                    break;
                }
            }
        }
        if scan.entrypoint.is_empty() {
            scan.entrypoint = "python main.py".to_string();
        }

        let has_test_signals = entries.contains(&"pytest.ini".to_string())
            || entries.contains(&"tox.ini".to_string())
            || dir.join("tests").exists()
            || entries
                .iter()
                .any(|e| e.starts_with("test_") && e.ends_with(".py"));
        if has_test_signals {
            scan.test_cmd = "python -m pytest tests/ -v".to_string();
        }
    } else {
        scan.entrypoint = "npm start".to_string();
        scan.test_cmd = "npm test".to_string();
    }

    scan
}
