use super::base::{Machine, MachineType};
use std::path::PathBuf;
use std::process::Command;

use crate::error::{CrustError, ExitCode};

/// Definition of LocalMachine with private fields.
pub struct LocalMachine {
    tmpdir: Option<String>,
}

/// Set of unique methods for this LocalMachine structure.
impl LocalMachine {
    pub fn new() -> Self {
        Self { tmpdir: None }
    }
}

/// Provided methods from trait to deliver a common interface.
impl Machine for LocalMachine {
    fn create_tmpdir(&mut self) -> String {
        let tmpdir = String::from_utf8(Command::new("mktemp").arg("-d").output().unwrap().stdout)
            .unwrap()
            .trim()
            .to_string();
        self.tmpdir = Some(tmpdir.clone());
        tmpdir
    }

    #[inline(always)]
    fn mtype(&self) -> MachineType {
        MachineType::LocalMachine
    }

    fn split(&self, size: u64, data: &str) -> Result<Vec<PathBuf>, CrustError> {
        Command::new("split")
            .arg("-b")
            .arg(size.to_string().as_str())
            .arg(data)
            .arg(format!(
                "{}/chunk_",
                self.tmpdir
                    .as_ref()
                    .expect("There is no tmp directory. Call `create_tmpdir` first.")
            ))
            .status()
            .expect("Error with splitting data");

        let output = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "ls {}/chunk_*",
                self.tmpdir
                    .as_ref()
                    .expect("There is no tmp directory. Call `create_tmpdir` first.")
            ))
            .output()?;

        let binding = String::from_utf8(output.stdout)?;

        let result: Vec<String> = binding
            .split('\n')
            .collect::<Vec<&str>>()
            .iter()
            .filter(|&v| !v.is_empty())
            .map(|v| v.to_string())
            .collect();

        let vec_of_paths: Vec<PathBuf> = result.into_iter().map(PathBuf::from).collect();

        Ok(vec_of_paths)
    }

    fn merge(&self, dst: &str) -> Result<(), CrustError> {
        self.exec(
            format!(
                "cat {}/chunk_* > {}",
                self.tmpdir
                    .as_ref()
                    .expect("There is no tmp directory. Call `create_tmpdir` first."),
                dst
            )
            .as_str(),
        )?;
        Ok(())
    }

    fn exec(&self, cmd: &str) -> Result<String, CrustError> {
        let result = Command::new("sh").arg("-c").arg(cmd).output()?;

        if !result.status.success() {
            return Err(CrustError {
                code: ExitCode::Local,
                message: String::from_utf8(result.stderr)?,
            });
        }

        Ok(String::from_utf8(result.stdout)?)
    }

    fn ssh_address(&self) -> String {
        "".to_string()
    }

    fn get_tmpdir(&self) -> String {
        self.tmpdir
            .as_ref()
            .expect("There is no tmp directory. Call `create_tmpdir` first.")
            .clone()
    }
}

/// Destructur implemtation for cleanup temporary directory when
/// struct leaves scope.
impl Drop for LocalMachine {
    fn drop(&mut self) {
        if let Some(tmp) = self.tmpdir.as_ref() {
            let _ = self.exec(format!("rm -r {}", tmp).as_str());
        }
    }
}

/// Default LocalMachine - never used, but clippy suggests
/// adding it in case someone else changes something.
impl Default for LocalMachine {
    fn default() -> Self {
        LocalMachine::new()
    }
}
