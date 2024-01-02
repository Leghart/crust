use std::path::PathBuf;

use super::base::{Machine, MachineType};
use crate::connection::SshConnection;

use crate::error::CrustError;

pub struct RemoteMachine {
    tmpdir: Option<String>,
    ssh: SshConnection,
}

impl RemoteMachine {
    pub fn new(
        user: String,
        host: String,
        password: Option<String>,
        pkey: Option<PathBuf>,
        port: u16,
    ) -> Self {
        let ssh = SshConnection::new(user, host, pkey, password, port);

        Self { ssh, tmpdir: None }
    }

    /// Run command on remote machine with private attribute -
    /// ssh connection.
    pub fn exec(&self, cmd: &str) -> Result<String, CrustError> {
        self.ssh.exec(cmd)
    }

    pub fn connect(&mut self) -> Result<(), CrustError> {
        self.ssh.connect()
    }
}

/// Provided methods from trait to deliver a common interface.
impl Machine for RemoteMachine {
    fn create_tmpdir(&mut self) -> String {
        let tmpdir = self.ssh.exec("mktemp -d").unwrap().trim().to_string();
        self.tmpdir = Some(tmpdir.clone());
        tmpdir
    }

    #[inline(always)]
    fn mtype(&self) -> MachineType {
        MachineType::RemoteMachine
    }

    fn split(&self, size: u64, data: &str) -> Result<Vec<PathBuf>, CrustError> {
        let cmd = format!(
            "split -b {} {} {}/chunk_",
            size,
            data,
            self.tmpdir
                .as_ref()
                .expect("There is no tmp directory. Call `create_tmpdir` first.")
        );

        self.exec(cmd.as_str())?;

        let cmd = format!(
            "ls {}/chunk_*",
            self.tmpdir
                .as_ref()
                .expect("There is no tmp directory. Call `create_tmpdir` first.")
        );
        let binding = self.exec(cmd.as_str())?;

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
        self.ssh.exec(
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
        self.exec(cmd)
    }

    fn ssh_address(&self) -> String {
        self.ssh.ssh_address()
    }

    fn get_tmpdir(&self) -> String {
        self.tmpdir
            .as_ref()
            .expect("There is no tmp directory. Call `create_tmpdir` first.")
            .clone()
    }
}

impl Drop for RemoteMachine {
    fn drop(&mut self) {
        if let Some(tmp) = self.tmpdir.as_ref() {
            let _ = self.exec(format!("rm -r {}", tmp).as_str());
        }
    }
}
