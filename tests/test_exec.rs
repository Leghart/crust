use assert_cmd;
use serial_test::serial;

extern crate test_utils;

#[test]
#[serial]
fn test_exec_on_remote() {
    let mut cmd = assert_cmd::Command::cargo_bin("crust").unwrap();
    cmd.args(&[
        "exec",
        "whoami",
        "--addr-to",
        "test_user@10.10.10.10",
        "--password-to",
        "1234",
    ]);

    cmd.assert().success();
    cmd.assert()
        .stdout("CrustResult { stdout: \"test_user\\n\", stderr: \"\", retcode: 0 }\n");
}
