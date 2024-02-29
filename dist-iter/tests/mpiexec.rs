#![feature(path_file_prefix)]
use std::{
    fs,
    path::PathBuf,
    process::{Command, ExitCode, Stdio},
};

use termcolor::Color;

mod utils;
use utils::term;

macro_rules! test {
    ($name:expr) => {{
        log!("test ");
        term::bold();
        log!("{}", $name);
        term::reset();
        log!(" ... ");
    }};
}

macro_rules! test_success {
    () => {{
        term::color(Color::Green);
        log!("ok\n");
        term::reset();
    }};
}

macro_rules! test_fail {
    ($($args:tt)*) => {{
        term::color(Color::Red);
        log!("FAILED\n");
        term::reset();
        log!($($args)*);
        log!("\n");
    }};
}

#[test]
fn mpiexec() -> ExitCode {
    let status = Command::new("which")
        .arg("mpiexec")
        .stdout(Stdio::null())
        .status()
        .unwrap();

    if !status.success() {
        println!("can not find mpiexec");
        return ExitCode::FAILURE;
    }

    let mut exit = ExitCode::SUCCESS;
    for file in fs::read_dir("tests/mpiexec_tests").unwrap() {
        let path = file.unwrap().path();
        test!(path.to_str().unwrap());
        let file_prefix = path.file_prefix().unwrap();

        let mut new_path = PathBuf::from("tests");
        new_path.push(path.file_name().unwrap());

        fs::copy(&path, &new_path).unwrap();

        let mut cmd = Command::new("cargo");
        cmd.arg("build")
            .arg("--target-dir")
            .arg("../target/tests/mpiexec")
            .arg("--test")
            .arg(file_prefix);

        let output = cmd.output().unwrap();

        fs::remove_file(new_path).unwrap();

        if !output.status.success() {
            test_fail!(
                "failed to compile test {path:?}:\n{}",
                std::str::from_utf8(&output.stderr).unwrap()
            );
            exit = ExitCode::FAILURE;
            continue;
        }

        let test_bin = fs::read_dir("../target/tests/mpiexec/debug/deps/")
            .unwrap()
            .find(|file| {
                if let Ok(file) = file {
                    let filename = file.file_name();
                    let filename = filename.to_str().unwrap();
                    filename.starts_with(file_prefix.to_str().unwrap()) && !filename.ends_with(".d")
                } else {
                    false
                }
            })
            .unwrap()
            .unwrap()
            .path();

        let mut cmd = Command::new("mpiexec");
        cmd.arg("--np")
            .arg("4")
            .arg("--oversubscribe")
            .arg(test_bin);

        let output = cmd.output().unwrap();

        if !output.status.success() {
            test_fail!(
                "mpiexec failed:\n{}\n{}",
                std::str::from_utf8(&output.stdout).unwrap(),
                std::str::from_utf8(&output.stderr).unwrap()
            );
            exit = ExitCode::FAILURE;
            continue;
        }
        test_success!();
    }

    log!("\n\n");

    exit
}
