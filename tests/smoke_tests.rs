use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates;
use predicates::prelude::*;

use std::fs;
use std::path::Path;

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
    let temp_dir = TempDir::new().expect("unable to create temp dir in smoke test");
    let temp_dir_path = temp_dir.path();

    assert!(!temp_dir_path.join("herm").is_dir());

    let test_home_dir = TempDir::new().expect("unable to create temp home dir in smoke test");

    let mut cmd = Command::cargo_bin("herm").unwrap();
    cmd.arg("list")
        .env("XDG_DATA_HOME", &temp_dir_path)
        .env("HOME", &test_home_dir.path())
        .assert()
        .append_context("main", "list")
        .success()
        .stdout(predicates::str::contains("Displaying: 0"));

    assert!(temp_dir_path.join("herm").is_dir());
}

#[test]
fn smoke_test_init_invocation() {
    let hermione_yml_path = Path::new("hermione.yml");

    assert!(!hermione_yml_path.is_file());

    let mut cmd = Command::cargo_bin("herm").unwrap();
    cmd.arg("init")
        .assert()
        .append_context("main", "init")
        .success();

    assert!(hermione_yml_path.is_file());

    fs::remove_file(hermione_yml_path)
        .expect("unable to clean up hermione.yml file after smoke test");
}

#[test]
fn smoke_test_install_example_package() {
    let temp_dir = TempDir::new().expect("unable to create temp dir in smoke test");
    let temp_dir_path = temp_dir.path();
    temp_dir
        .child("herm")
        .child("example-package")
        .assert(predicate::path::missing());

    let test_home_dir = TempDir::new().expect("unable to create temp home dir in smoke test");

    let mut cmd = Command::cargo_bin("herm").unwrap();
    cmd.arg("install")
        .arg("example-package")
        .env("XDG_DATA_HOME", &temp_dir_path)
        .env("HOME", &test_home_dir.path())
        .assert()
        .append_context("main", "install example-package")
        .success();

    temp_dir
        .child("herm")
        .child("example-package")
        .assert(predicate::path::is_dir());

    test_home_dir
        .child("bamboo.txt")
        .assert(predicate::path::exists());
}

#[test]
fn smoke_test_post_install_hook() {
    let temp_dir = TempDir::new().expect("unable to create temp dir in smoke test");
    let temp_dir_path = temp_dir.path();
    temp_dir
        .child("herm")
        .child("example-package")
        .assert(predicate::path::missing());

    let test_home_dir = TempDir::new().expect("unable to create temp home dir in smoke test");

    let mut cmd = Command::cargo_bin("herm").unwrap();
    cmd.arg("install")
        .arg("example-package")
        .env("XDG_DATA_HOME", &temp_dir_path)
        .env("HOME", &test_home_dir.path())
        .assert()
        .append_context("main", "install example-package")
        .stdout(predicate::str::contains("Hello from post_install hook"));
}
