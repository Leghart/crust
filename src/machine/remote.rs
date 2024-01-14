use std::cell::RefCell;
use std::path::PathBuf;

use super::base::{Machine, MachineType};

use crate::connection::{SshConnection, SSH};
use crate::error::CrustError;
use crate::exec::Exec;
use crate::interfaces::tmpdir::TemporaryDirectory;
use crate::tscp::Tscp;

/// Definition of RemoteMachine with private fields.
/// - tmpdir: possible path to temporary directory
/// - should_remove_tmpdir: determines whether dir
///   should be removed on dropping object
/// - ssh: reference to `SshConnection` object which
///   provides access to remote servers.
pub struct RemoteMachine {
    tmpdir: Option<String>,
    should_remove_tmpdir: bool,
    ssh: RefCell<SshConnection>,
}

/// Set of unique methods for this RemoteMachine structure.
impl RemoteMachine {
    pub fn new(
        user: String,
        host: String,
        password: Option<String>,
        pkey: Option<PathBuf>,
        port: u16,
    ) -> Self {
        let ssh = SshConnection::new(user, host, pkey, password, port);

        Self {
            ssh: RefCell::new(ssh),
            tmpdir: None,
            should_remove_tmpdir: true,
        }
    }

    /// Creates a connection to a remote server on demand.
    pub fn connect(&mut self) -> Result<(), CrustError> {
        self.ssh.borrow_mut().connect()
    }
}

/// Provides methods from Machine trait to deliver a common interface.
impl Machine for RemoteMachine {
    #[inline(always)]
    fn mtype(&self) -> MachineType {
        MachineType::RemoteMachine
    }

    #[inline(always)]
    fn ssh_address(&self) -> String {
        self.ssh.borrow().ssh_address()
    }

    fn get_session(&self) -> Option<ssh2::Session> {
        Some(self.ssh.borrow().session().clone())
    }
}

/// Implementation of temporary directory handling.
impl TemporaryDirectory for RemoteMachine {
    fn can_be_removed(&self) -> bool {
        self.should_remove_tmpdir
    }

    fn get_tmpdir(&self) -> String {
        self.tmpdir
            .clone()
            .expect("Temporary directory was not created")
    }

    fn tmpdir_exists(&self) -> bool {
        self.tmpdir.is_some()
    }

    fn create_tmpdir(&mut self) {
        self.tmpdir = Some(
            self.exec("mktemp -d")
                .expect("Can not create temporary directory")
                .trim()
                .to_string(),
        )
    }

    fn remove_tmpdir(&self) {
        if self.can_be_removed() {
            let _ = self.exec(format!("rm -r {}", self.get_tmpdir()).as_str());
        }
    }
}

/// Add `execute` method for RemoteMachine
impl Exec for RemoteMachine {
    /// Exec command on remote machine. If connection
    /// was not established (or this is a first call), connect
    /// with ssh first.
    fn exec(&self, cmd: &str) -> Result<String, CrustError> {
        if !self.ssh.borrow().is_connected() {
            self.ssh.borrow_mut().connect()?;
        }
        self.ssh.borrow().execute(cmd)
    }
}

/// Add `tscp` method for RemoteMachine
impl Tscp for RemoteMachine {
    fn split(&mut self, size: u64, data: &str) -> Result<Vec<PathBuf>, CrustError> {
        let cmd = format!("split -b {} {} {}/chunk_", size, data, self.get_tmpdir());
        self.exec(cmd.as_str())?;

        let cmd = format!("ls {}/chunk_*", self.get_tmpdir());
        let binding = self.exec(cmd.as_str())?;

        self._string_chunks_to_vec(binding)
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

    #[inline(always)]
    fn get_address(&self) -> String {
        self.ssh_address()
    }

    #[inline(always)]
    fn get_machine(&self) -> MachineType {
        self.mtype()
    }
}

/// Destructur implemtation for cleanup temporary directory when
/// struct leaves scope.
impl Drop for RemoteMachine {
    fn drop(&mut self) {
        if self.tmpdir_exists() {
            self.remove_tmpdir();
        }
    }
}

/// Custom Clone implementation that guarantees that
/// copies of the object will not delete the directory.
impl Clone for RemoteMachine {
    fn clone(&self) -> Self {
        RemoteMachine {
            tmpdir: self.tmpdir.clone(),
            should_remove_tmpdir: false,
            ssh: self.ssh.clone(),
        }
    }
}
