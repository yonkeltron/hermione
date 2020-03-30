use assert_cmd::Command;

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
