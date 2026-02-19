use assert_cmd::Command;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("mlvm").unwrap();
    cmd.arg("--help")
        .assert()
        .success();
}

#[test]
fn test_node_list_remote() {
    let mut cmd = Command::cargo_bin("mlvm").unwrap();
    cmd.arg("node")
        .arg("list-remote")
        .assert()
        .success(); // This might fail if no internet, but verifies the command runs
}