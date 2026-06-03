use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn version_flag() {
    Command::cargo_bin("luna")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("luna 0.1.0"));
}

#[test]
fn help_flag() {
    Command::cargo_bin("luna")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Luna monorepo CLI"))
        .stdout(predicate::str::contains("build"))
        .stdout(predicate::str::contains("dev"))
        .stdout(predicate::str::contains("outdated"))
        .stdout(predicate::str::contains("update"));
}

#[test]
fn subcommand_help() {
    Command::cargo_bin("luna")
        .unwrap()
        .arg("build")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--affected"));
}

#[test]
fn update_help() {
    Command::cargo_bin("luna")
        .unwrap()
        .arg("update")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--major"));
}

#[test]
fn missing_subcommand() {
    Command::cargo_bin("luna")
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains("COMMAND"));
}

#[test]
fn binary_name_alias() {
    Command::cargo_bin("l")
        .unwrap()
        .args(["check", "--help"])
        .assert()
        .success();

    Command::cargo_bin("ln")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("luna 0.1.0"));
}
