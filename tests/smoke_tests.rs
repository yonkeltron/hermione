use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn smoke_test_naked_invocation() {
    let mut cmd = Command::cargo_bin("herm").unwrap();
    cmd.assert().append_context("main", "no args").failure();
}

#[test]
fn smoke_test_help_invocation() {
    let mut cmd = Command::cargo_bin("herm").unwrap();
    cmd.arg("help")
        .assert()
        .append_context("main", "help")
        .success();
}

#[test]
fn smoke_test_list_invocation() {
    let mut cmd = Command::cargo_bin("herm").unwrap();
    cmd.arg("list")
        .assert()
        .append_context("main", "list")
        .stdout(predicates::str::contains("Displaying"));
}
