use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

#[derive(Debug, Clone, PartialEq)]
enum Op {
    Ge,
    Gt,
    Le,
    Lt,
    Eq,
}

#[derive(Debug, Clone)]
struct Constraint {
    op: Op,
    version: Version,
}

#[derive(Deserialize)]
struct VersionsFile {
    python: std::collections::HashMap<String, std::collections::HashMap<String, RuntimeEntry>>,
    node: std::collections::HashMap<String, std::collections::HashMap<String, RuntimeEntry>>,
}

#[derive(Deserialize)]
struct RuntimeEntry {
    url: String,
    sha256: String,
}

pub struct OsInfo {
    pub os: String,
    pub arch: String,
}

pub struct SystemRuntime {
    pub version: String,
    pub path: PathBuf,
}

pub struct RuntimeDownload {
    pub name: String,
    pub version: String,
    pub url: String,
    pub sha256: String,
    pub dest: PathBuf,
}

pub struct PythonCheck {
    pub system: Option<SystemRuntime>,
    pub download: Option<RuntimeDownload>,
}

pub struct NodeCheck {
    pub system: Option<SystemRuntime>,
    pub download: Option<RuntimeDownload>,
}

pub fn detect_os() -> OsInfo {
    let os = match std::env::consts::OS {
        "macos" => "macos",
        "linux" => "linux",
        _ => "linux",
    };
    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        _ => "x64",
    };
    OsInfo {
        os: os.to_string(),
        arch: arch.to_string(),
    }
}

pub fn check_python(constraint_str: &str) -> anyhow::Result<PythonCheck> {
    let constraints = parse_constraint(constraint_str)?;
    if let Some((version, path)) = find_system_python()
        && version_satisfies(&version, &constraints)
    {
        return Ok(PythonCheck {
            system: Some(SystemRuntime {
                version: format_version(&version),
                path,
            }),
            download: None,
        });
    }
    let os = detect_os();
    let download = find_best_download("python", &constraints, &os)?;
    Ok(PythonCheck {
        system: None,
        download,
    })
}

pub fn check_node(constraint_str: &str) -> anyhow::Result<NodeCheck> {
    let constraints = parse_constraint(constraint_str)?;
    if let Some((version, path)) = find_system_node()
        && version_satisfies(&version, &constraints)
    {
        return Ok(NodeCheck {
            system: Some(SystemRuntime {
                version: format_version(&version),
                path,
            }),
            download: None,
        });
    }
    let os = detect_os();
    let download = find_best_download("node", &constraints, &os)?;
    Ok(NodeCheck {
        system: None,
        download,
    })
}

fn find_system_python() -> Option<(Version, PathBuf)> {
    let path = which::which("python3").ok()?;
    let output = Command::new(&path).arg("--version").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let version_str = if stdout.is_empty() {
        stderr.trim()
    } else {
        stdout.trim()
    };
    let version = parse_python_output(version_str).ok()?;
    Some((version, path))
}

fn find_system_node() -> Option<(Version, PathBuf)> {
    let path = which::which("node").ok()?;
    let output = Command::new(&path).arg("--version").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version_str = stdout.trim();
    let version = parse_node_output(version_str).ok()?;
    Some((version, path))
}

fn parse_python_output(raw: &str) -> anyhow::Result<Version> {
    let raw = raw.strip_prefix("Python ").unwrap_or(raw);
    parse_version(raw)
}

fn parse_node_output(raw: &str) -> anyhow::Result<Version> {
    let raw = raw.strip_prefix('v').unwrap_or(raw);
    parse_version(raw)
}

fn parse_constraint(input: &str) -> anyhow::Result<Vec<Constraint>> {
    let mut constraints = Vec::new();
    for part in input.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (op, ver_str) = if let Some(rest) = part.strip_prefix(">=") {
            (Op::Ge, rest.trim())
        } else if let Some(rest) = part.strip_prefix("<=") {
            (Op::Le, rest.trim())
        } else if let Some(rest) = part.strip_prefix(">") {
            (Op::Gt, rest.trim())
        } else if let Some(rest) = part.strip_prefix("<") {
            (Op::Lt, rest.trim())
        } else if let Some(rest) = part.strip_prefix("==") {
            (Op::Eq, rest.trim())
        } else {
            anyhow::bail!("invalid version constraint: {part}");
        };
        let version = parse_version(ver_str)?;
        constraints.push(Constraint { op, version });
    }
    Ok(constraints)
}

fn parse_version(s: &str) -> anyhow::Result<Version> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.is_empty() || parts.len() > 3 {
        anyhow::bail!("invalid version: {s}");
    }
    let major = parts[0]
        .parse::<u32>()
        .with_context(|| format!("invalid major version in: {s}"))?;
    let minor = if parts.len() > 1 {
        parts[1].parse::<u32>().unwrap_or(0)
    } else {
        0
    };
    let patch = if parts.len() > 2 {
        parts[2].parse::<u32>().unwrap_or(0)
    } else {
        0
    };
    Ok(Version {
        major,
        minor,
        patch,
    })
}

fn version_satisfies(version: &Version, constraints: &[Constraint]) -> bool {
    for c in constraints {
        let ok = match c.op {
            Op::Ge => version >= &c.version,
            Op::Gt => version > &c.version,
            Op::Le => version <= &c.version,
            Op::Lt => version < &c.version,
            Op::Eq => version == &c.version,
        };
        if !ok {
            return false;
        }
    }
    true
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
    }
}

fn format_version(v: &Version) -> String {
    format!("{}.{}.{}", v.major, v.minor, v.patch)
}

fn find_best_download(
    name: &str,
    constraints: &[Constraint],
    os: &OsInfo,
) -> anyhow::Result<Option<RuntimeDownload>> {
    let data: VersionsFile =
        toml::from_str(include_str!("../data/versions.toml")).context("invalid versions.toml")?;
    let platform_key = format!("{}_{}", os.os, os.arch);
    let versions_map: &std::collections::HashMap<
        String,
        std::collections::HashMap<String, RuntimeEntry>,
    > = match name {
        "python" => &data.python,
        "node" => &data.node,
        _ => return Ok(None),
    };
    let cache_root = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("~/.cache"))
        .join("small")
        .join("runtimes");
    for (ver_str, platforms) in versions_map {
        let version = parse_version(ver_str)?;
        if version_satisfies(&version, constraints)
            && let Some(entry) = platforms.get(&platform_key)
        {
            return Ok(Some(RuntimeDownload {
                name: name.to_string(),
                version: ver_str.clone(),
                url: entry.url.clone(),
                sha256: entry.sha256.clone(),
                dest: cache_root.join(name).join(ver_str),
            }));
        }
    }
    Ok(None)
}

pub fn download_and_extract(
    url: &str,
    sha256_expected: &str,
    dest: &std::path::Path,
) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        if dest.join("bin").exists() {
            return Ok(());
        }
        let parent = dest.parent().unwrap_or(dest);
        std::fs::create_dir_all(parent)?;
        let tmp = parent.join(format!(".tmp_download_{}", std::process::id()));

        let response = reqwest::get(url).await?.error_for_status()?;
        let bytes = response.bytes().await?;

        if sha256_expected != "0000000000000000000000000000000000000000000000000000000000000000" {
            use sha2::Digest;
            let mut hasher = sha2::Sha256::new();
            hasher.update(&bytes);
            let hash = format!("{:x}", hasher.finalize());
            if hash != sha256_expected {
                return Err(anyhow::anyhow!(
                    "SHA256 mismatch for {url}\n  expected: {sha256_expected}\n  got:      {hash}"
                ));
            }
        }

        std::fs::write(&tmp, &bytes)?;

        let file = std::fs::File::open(&tmp)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        std::fs::create_dir_all(dest)?;
        archive.unpack(dest)?;
        std::fs::remove_file(&tmp)?;

        Ok::<_, anyhow::Error>(())
    })
}
