use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, ensure, Result};
use clap::Parser;
use walkdir::WalkDir;
use xshell::{cmd, Shell};

#[derive(Parser)]
pub struct Args {
    #[arg(short, long, env = "NOIR_LIBS_ARCHIVE")]
    archive: PathBuf,
    #[arg(short, long, env = "EXPECTED_VERSION")]
    expected_version: String,
}

pub fn main(args: Args) -> Result<()> {
    let sh = Shell::new()?;

    let expected_version = args.expected_version.trim_start_matches('v');

    let install_dir = sh.create_temp_dir()?;
    if args.archive.file_name().unwrap().to_string_lossy().ends_with(".tar.gz") {
        let archive = &args.archive;
        let install_dir = install_dir.path();
        cmd!(sh, "tar -zxvf {archive} -C {install_dir}").run()?;
    } else {
        let archive = &args.archive;
        let install_dir = install_dir.path();
        cmd!(sh, "7z x -y {archive} -o{install_dir}").run()?;
    }

    let noir_libs = find_noir_libs_binary(install_dir.path())?;

    cmd!(sh, "noir-libs --version").run()?;
    cmd!(sh, "noir-libs --help").run()?;

    let noir_libs_version = cmd!(sh, "noir-libs -V").read()?;
    ensure!(
        noir_libs_version.contains(expected_version),
        "wrong noir-libs version, expected: {expected_version}, got: {noir_libs_version}",
    );

    let workdir = sh.create_temp_dir()?;
    sh.change_dir(workdir.path());
    cmd!(sh, "noir-libs new smoke_test --test-runner=cairo-test").run()?;
    sh.change_dir(workdir.path().join("smoke_test"));
    cmd!(sh, "noir-libs build").run()?;
    cmd!(sh, "noir-libs test").run()?;

    Ok(())
}

fn find_noir_libs_binary(install_dir: &Path) -> Result<PathBuf> {
    for e in WalkDir::new(install_dir) {
        let e = e?;
        if e.file_type().is_file()
            && e.file_name().to_string_lossy() == format!("noir-libs{}", env::consts::EXE_SUFFIX)
        {
            return Ok(e.into_path());
        }
    }

    Err(anyhow!(
        "could not find noir-libs{} executable in the archive",
        env::consts::EXE_SUFFIX
    ))
}
