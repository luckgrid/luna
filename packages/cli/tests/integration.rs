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
fn missing_subcommand() {
    Command::cargo_bin("luna")
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains("COMMAND"));
}

// --- Subcommand help checks (verify clap definitions survived refactor) ---

macro_rules! subcommand_help_test {
    ($name:ident, $subcmd:expr, $needle:expr) => {
        #[test]
        fn $name() {
            Command::cargo_bin("luna")
                .unwrap()
                .arg($subcmd)
                .arg("--help")
                .assert()
                .success()
                .stdout(predicate::str::contains($needle));
        }
    };
}

subcommand_help_test!(build_help, "build", "--affected");
subcommand_help_test!(test_help, "test", "--affected");
subcommand_help_test!(dev_help, "dev", "project");
subcommand_help_test!(start_help, "start", "project");
subcommand_help_test!(run_help, "run", "targets");
subcommand_help_test!(graph_help, "graph", "Emit the project graph");
subcommand_help_test!(tasks_help, "tasks", "tasks");
subcommand_help_test!(projects_help, "projects", "projects");
subcommand_help_test!(ci_help, "ci", "ci");
subcommand_help_test!(lint_help, "lint", "--fix");
subcommand_help_test!(format_help, "format", "--check");
subcommand_help_test!(typecheck_help, "typecheck", "typecheck");
subcommand_help_test!(check_help, "check", "format check + typecheck");
subcommand_help_test!(fix_help, "fix", "Lint fix + format");
subcommand_help_test!(outdated_help, "outdated", "outdated");
subcommand_help_test!(update_help, "update", "--major");
subcommand_help_test!(install_help, "install", "--workspace");
subcommand_help_test!(clean_help, "clean", "reset");

#[test]
fn run_requires_targets() {
    Command::cargo_bin("luna")
        .unwrap()
        .arg("run")
        .assert()
        .failure()
        .stderr(predicate::str::contains("<TARGETS>"));
}

#[test]
fn verbose_flag_with_subcommand() {
    Command::cargo_bin("luna")
        .unwrap()
        .arg("-v")
        .arg("tasks")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn quiet_flag_with_subcommand() {
    Command::cargo_bin("luna")
        .unwrap()
        .arg("-q")
        .arg("projects")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn firewall_flag_visible_in_help() {
    Command::cargo_bin("luna")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("firewall"));
}
