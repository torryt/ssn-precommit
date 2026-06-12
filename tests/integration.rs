use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::TempDir;

/// Creates a temp git repo and returns the TempDir handle (keeps it alive).
fn setup_repo() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path();

    StdCommand::new("git")
        .args(["init"])
        .current_dir(path)
        .output()
        .expect("git init failed");

    StdCommand::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(path)
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(path)
        .output()
        .unwrap();

    dir
}

/// Stage a file with given content.
fn stage_file(dir: &TempDir, name: &str, content: &str) {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("failed to write file");
    StdCommand::new("git")
        .args(["add", name])
        .current_dir(dir.path())
        .output()
        .expect("git add failed");
}

#[test]
fn no_files_exits_success() {
    Command::cargo_bin("ssn-precommit")
        .unwrap()
        .assert()
        .success();
}

#[test]
fn detects_ssn_in_staged_file() {
    let dir = setup_repo();
    stage_file(&dir, "secret.txt", "My SSN is 12345678901\n");

    Command::cargo_bin("ssn-precommit")
        .unwrap()
        .current_dir(dir.path())
        .arg("secret.txt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("123456*****"));
}

#[test]
fn no_mask_shows_full_ssn() {
    let dir = setup_repo();
    stage_file(&dir, "secret.txt", "12345678901\n");

    Command::cargo_bin("ssn-precommit")
        .unwrap()
        .current_dir(dir.path())
        .args(["--no-mask", "secret.txt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("12345678901"));
}

#[test]
fn ignore_equals_syntax() {
    let dir = setup_repo();
    stage_file(&dir, "secret.txt", "12345678901\n");

    Command::cargo_bin("ssn-precommit")
        .unwrap()
        .current_dir(dir.path())
        .args(["--ignore=12345678901", "secret.txt"])
        .assert()
        .success();
}

#[test]
fn ignore_space_syntax() {
    let dir = setup_repo();
    stage_file(&dir, "secret.txt", "12345678901\n");

    Command::cargo_bin("ssn-precommit")
        .unwrap()
        .current_dir(dir.path())
        .args(["--ignore", "12345678901", "secret.txt"])
        .assert()
        .success();
}

#[test]
fn ssnignore_file() {
    let dir = setup_repo();
    stage_file(&dir, "secret.txt", "12345678901\n");
    fs::write(dir.path().join(".ssnignore"), "# comment\n12345678901\n").unwrap();

    Command::cargo_bin("ssn-precommit")
        .unwrap()
        .current_dir(dir.path())
        .arg("secret.txt")
        .assert()
        .success();
}

#[test]
fn detects_spaced_ssn_format() {
    let dir = setup_repo();
    stage_file(&dir, "spaced.txt", "ID: 123456 78901\n");

    Command::cargo_bin("ssn-precommit")
        .unwrap()
        .current_dir(dir.path())
        .arg("spaced.txt")
        .assert()
        .failure();
}

#[test]
fn clean_file_exits_success() {
    let dir = setup_repo();
    stage_file(&dir, "clean.txt", "Hello world, nothing here\n");

    Command::cargo_bin("ssn-precommit")
        .unwrap()
        .current_dir(dir.path())
        .arg("clean.txt")
        .assert()
        .success();
}

#[test]
fn unstaged_changes_not_detected() {
    let dir = setup_repo();
    stage_file(&dir, "file.txt", "safe content\n");

    // Commit so it's tracked
    StdCommand::new("git")
        .args(["commit", "-m", "init", "--no-verify"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    // Modify with SSN but don't stage
    fs::write(dir.path().join("file.txt"), "12345678901\n").unwrap();

    Command::cargo_bin("ssn-precommit")
        .unwrap()
        .current_dir(dir.path())
        .arg("file.txt")
        .assert()
        .success();
}

#[test]
fn ignore_comma_separated_list() {
    let dir = setup_repo();
    stage_file(&dir, "multi.txt", "11111111111\n22222222222\n");

    Command::cargo_bin("ssn-precommit")
        .unwrap()
        .current_dir(dir.path())
        .args(["--ignore=11111111111,22222222222", "multi.txt"])
        .assert()
        .success();
}
