use std::{process::Command, env, path::PathBuf};
use anyhow::{self, Context};

fn main() {
    let output = Command::new("git")
        .args(&["describe", "--always"])
        .output().context("Failed to get git version")
        .unwrap();
    let git_version = String::from_utf8(output.stdout).unwrap();
    println!("Using GIT_VERSION={}", git_version);
    println!("cargo:rustc-env=GIT_VERSION={}", git_version);

    compile_empty_cart().expect("Failed to create empty cart");    
}


const MAPPER_CONFIG: &str = "nrom_32k_vert.cfg";

fn compile_empty_cart() -> anyhow::Result<()> {

    let cc65_home: PathBuf = env::var("CC65_HOME").context("CC65_HOME is not valid")?.into();
    let cc65_bin = cc65_home.join("bin");
    let cc65 = cc65_bin.join("cc65");
    let ca65 = cc65_bin.join("ca65");
    let ld65 = cc65_bin.join("ld65");

    Command::new(cc65)
        .current_dir("empty")
        .args(&["-Oirs", "empty.c", "--add-source"])
        .spawn()?.wait().context("").and_then(exit_ok).context("Failed to compile empty cart")?;

    Command::new(&ca65)
        .current_dir("empty")
        .args(&["crt0.s"])
        .spawn()?.wait().context("").and_then(exit_ok).context("Failed to assemble empty cart prelude")?;

    Command::new(&ca65)
        .current_dir("empty")
        .args(&["-g", "empty.s"])
        .spawn()?.wait().context("").and_then(exit_ok).context("Failed to assemble empty cart")?;

    Command::new(ld65)
        .current_dir("empty")
        .args(&[
            "--config", MAPPER_CONFIG, 
            "-o", "empty.nes",
            "crt0.o",
            "empty.o",
            "nes.lib",
            "-Ln", "labels.txt",
            "--dbgfile", "dbg.txt",
        ])
        .spawn()?.wait().context("").and_then(exit_ok).context("Failed to link empty cart")?;

    Ok(())
}

fn exit_ok(status: std::process::ExitStatus) -> anyhow::Result<()> {
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::format_err!("Exited with status {}", status.code().unwrap_or(-1)))
    }
}