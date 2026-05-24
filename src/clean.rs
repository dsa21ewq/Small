use std::fs;

use crate::display;

pub fn run() -> anyhow::Result<()> {
    remove_artifacts();
    display::info("Cleaned project artifacts. Runtime cache preserved.");
    Ok(())
}

pub fn remove_artifacts() {
    if fs::remove_dir_all(".small_venv").is_ok() {
        display::info("Removed .small_venv");
    }
    if fs::remove_dir_all("node_modules").is_ok() {
        display::info("Removed node_modules");
    }
}
