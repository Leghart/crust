use assert_cmd;
use serial_test::serial;
use test_utils::{exec_on_local, exec_on_remote};

extern crate test_utils;

#[serial]
#[test]
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

#[serial]
#[test]
fn test_exec_on_local() {
    let mut cmd = assert_cmd::Command::cargo_bin("crust").unwrap();
    cmd.args(&["exec", "echo 'test'"]);

    cmd.assert().success();
    cmd.assert()
        .stdout("CrustResult { stdout: \"test\\n\", stderr: \"\", retcode: 0 }\n");
}

#[ignore = "toggling"]
#[serial]
#[test]
fn test_exec_rt_not_merged_streams_on_local() {
    let mut cmd = assert_cmd::Command::cargo_bin("crust").unwrap();
    cmd.args(&["exec", "./test_utils/test_run_script.sh", "--rt"]);

    cmd.assert().success();
    cmd.assert()
        .stdout("STDOUT 1\nSTDOUT 2\nSTDERR 1\nSTDERR 2\n");
}

#[serial]
#[test]
fn test_exec_rt_merged_streams_on_local() {
    let mut cmd = assert_cmd::Command::cargo_bin("crust").unwrap();
    cmd.args(&["exec", "./test_utils/test_run_script.sh", "--rt", "--merge"]);

    cmd.assert().success();
    cmd.assert()
        .stdout("STDERR 1\nSTDOUT 1\nSTDERR 2\nSTDOUT 2\n");
}

#[cfg(not(feature = "CI"))]
#[serial]
#[test]
fn test_exec_rt_not_merged_streams_on_remote() {
    exec_on_local(
        "scp -i test_utils/rsa_keys/id_rsa test_utils/test_run_script.sh test_user@10.10.10.10:",
    );

    assert_eq!(
        exec_on_remote("ls test_run_script.sh"),
        "test_run_script.sh"
    );

    exec_on_remote("chmod +x test_run_script.sh");

    let mut cmd = assert_cmd::Command::cargo_bin("crust").unwrap();
    cmd.args(&[
        "exec",
        "./test_run_script.sh",
        "--rt",
        "--addr-to",
        "test_user@10.10.10.10",
        "--password-to",
        "1234",
    ]);

    cmd.assert().success();
    cmd.assert()
        .stdout("STDOUT 1\nSTDOUT 2\nSTDERR 1\nSTDERR 2\n\n");
}

#[cfg(not(feature = "CI"))]
#[serial]
#[test]
fn test_exec_rt_merged_streams_on_remote() {
    exec_on_local(
        "scp -i test_utils/rsa_keys/id_rsa test_utils/test_run_script.sh test_user@10.10.10.10:",
    );

    assert_eq!(
        exec_on_remote("ls test_run_script.sh"),
        "test_run_script.sh"
    );

    exec_on_remote("chmod +x test_run_script.sh");

    let mut cmd = assert_cmd::Command::cargo_bin("crust").unwrap();
    cmd.args(&[
        "exec",
        "./test_run_script.sh",
        "--rt",
        "--merge",
        "--addr-to",
        "test_user@10.10.10.10",
        "--password-to",
        "1234",
    ]);

    cmd.assert().success();
    cmd.assert()
        .stdout("STDERR 1\nSTDOUT 1\nSTDERR 2\nSTDOUT 2\n");
}
