#![feature(path_file_prefix)]
use std::{
    fs,
    path::PathBuf,
    process::{Command, ExitCode},
};

#[test]
fn mpiexec() -> ExitCode {
    let status = Command::new("which").arg("mpiexec").status().unwrap();

    if !status.success() {
        eprintln!("can not find mpiexec");
        return ExitCode::FAILURE;
    }

    let mut exit = ExitCode::SUCCESS;
    for file in fs::read_dir("tests/mpiexec_tests").unwrap() {
        let path = file.unwrap().path();
        let file_prefix = path.file_prefix().unwrap();

        let mut new_path = PathBuf::from("tests");
        new_path.push(path.file_name().unwrap());

        eprintln!("cp {path:?} -> {new_path:?}");
        fs::copy(&path, &new_path).unwrap();

        let mut cmd = Command::new("cargo");
        cmd.arg("build")
            .arg("--target-dir")
            .arg("../target/tests/mpiexec")
            .arg("--test")
            .arg(file_prefix);

        eprintln!("running {cmd:?}");
        let output = cmd.output().unwrap();

        eprintln!("rm {new_path:?}");
        fs::remove_file(new_path).unwrap();

        if !output.status.success() {
            eprintln!(
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

        eprintln!("found test bin {test_bin:?}");

        let mut cmd = Command::new("mpiexec");
        cmd.arg("--np")
            .arg("4")
            .arg("--oversubscribe")
            .arg(test_bin);

        eprintln!("running {cmd:?}",);
        let output = cmd.output().unwrap();

        if !output.status.success() {
            eprintln!(
                "mpiexec failed:\n{}\n{}",
                std::str::from_utf8(&output.stdout).unwrap(),
                std::str::from_utf8(&output.stderr).unwrap()
            );
            exit = ExitCode::FAILURE;
            continue;
        }
    }
    exit
}
