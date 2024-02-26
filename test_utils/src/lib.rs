use std::path::PathBuf;
use std::process::Command;

pub fn exec_on_local(cmd: &str) -> String {
    let result = Command::new("sh").arg("-c").arg(cmd).output().unwrap();

    String::from_utf8(result.stdout)
        .unwrap()
        .as_str()
        .trim()
        .to_string()
}

pub fn exec_on_remote(cmd: &str) -> String {
    let rsa_ssh = "test_utils/rsa_keys/id_rsa";
    let user = "test_user";
    let host = "10.10.10.10";
    Command::new("chmod")
        .arg("600")
        .arg(rsa_ssh)
        .output()
        .unwrap();

    let command = format!("ssh -o StrictHostKeyChecking=no -i {rsa_ssh} {user}@{host} {cmd}");

    let result = Command::new("sh")
        .arg("-c")
        .arg(command.as_str())
        .output()
        .unwrap();
    String::from_utf8(result.stdout)
        .unwrap()
        .as_str()
        .trim()
        .to_string()
}

pub fn exists_on_remote(path: PathBuf, dir: bool) -> bool {
    let flag = match dir {
        true => "-d",
        false => "-f",
    };
    let cmd = format!(
        "test {flag} {} && echo 'true' || echo 'false'",
        path.as_path().to_str().unwrap()
    );
    let result = exec_on_remote(&cmd);

    if result == "true" {
        return true;
    } else {
        return false;
    }
}
