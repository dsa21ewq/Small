use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pm {
    Brew,
    Apt,
}

#[derive(Deserialize)]
struct PackagesFile {
    #[serde(flatten)]
    entries: std::collections::HashMap<String, PackageEntry>,
}

#[derive(Deserialize)]
struct PackageEntry {
    brew: Option<String>,
    apt: Option<String>,
}

pub struct PkgCheck {
    pub logical: String,
    pub resolved: String,
    pub missing: bool,
}

pub fn detect_pm() -> Option<Pm> {
    if which::which("brew").is_ok() {
        return Some(Pm::Brew);
    }
    if which::which("apt-get").is_ok() {
        return Some(Pm::Apt);
    }
    None
}

pub fn check_packages(names: &[String], pm: &Pm) -> Vec<PkgCheck> {
    let data: PackagesFile =
        toml::from_str(include_str!("../data/packages.toml")).expect("valid packages.toml");
    names
        .iter()
        .map(|logical| {
            let resolved = data
                .entries
                .get(logical)
                .and_then(|e| match pm {
                    Pm::Brew => e.brew.clone(),
                    Pm::Apt => e.apt.clone(),
                })
                .unwrap_or_else(|| logical.clone());
            let missing = !is_installed(&resolved, pm);
            PkgCheck {
                logical: logical.clone(),
                resolved,
                missing,
            }
        })
        .collect()
}

fn is_installed(name: &str, pm: &Pm) -> bool {
    match pm {
        Pm::Brew => which::which(name).is_ok(),
        Pm::Apt => {
            if which::which(name).is_ok() {
                return true;
            }
            let result = std::process::Command::new("dpkg")
                .args(["-s", name])
                .output();
            result.map(|o| o.status.success()).unwrap_or(false)
        }
    }
}

pub fn install_packages(pm: &Pm, packages: &[String]) -> anyhow::Result<()> {
    match pm {
        Pm::Brew => {
            let mut args = vec!["install"];
            args.extend(packages.iter().map(|s| s.as_str()));
            duct::cmd("brew", args).run()?;
        }
        Pm::Apt => {
            let mut args = vec!["apt-get", "install", "-y"];
            args.extend(packages.iter().map(|s| s.as_str()));
            duct::cmd("sudo", args).run()?;
        }
    }
    Ok(())
}
