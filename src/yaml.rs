use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SmallYaml {
    pub project: Project,
    #[serde(default)]
    pub runtimes: Runtimes,
    #[serde(default)]
    pub system: Vec<String>,
    #[serde(default)]
    pub dependencies: Dependencies,
    pub requirements_file: Option<String>,
    pub package_file: Option<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub pre_install: Vec<String>,
    #[serde(default)]
    pub post_install: Vec<String>,
    pub test: Option<String>,
    pub entrypoint: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Project {
    pub name: String,
    pub language: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Runtimes {
    pub python: Option<String>,
    pub node: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Dependencies {
    #[serde(default)]
    pub python: Vec<String>,
    #[serde(default)]
    pub node: Vec<String>,
}

pub fn parse(path: &str) -> anyhow::Result<SmallYaml> {
    let content = std::fs::read_to_string(path)?;
    let config: SmallYaml = serde_yaml::from_str(&content)?;
    Ok(config)
}
