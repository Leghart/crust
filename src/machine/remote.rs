use std::cell::RefCell;
use std::path::PathBuf;

use super::base::{Machine, MachineType};

use crate::connection::manager::{MachinesManager, MachinesManagerMethods};
use crate::connection::{SshConnection, SSH};
use crate::error::{CrustError, ExitCode};
use crate::exec::Exec;
use crate::interfaces::tmpdir::TemporaryDirectory;
use crate::scp::Scp;
use uuid::Uuid;

/// Definition of RemoteMachine with private fields.
/// - id: machine id for MachinesManager
/// - tmpdir: possible path to temporary directory
/// - should_remove_tmpdir: determines whether dir
///   should be removed on dropping object
/// - ssh: reference to `SshConnection` object which
///   provides access to remote servers.
pub struct RemoteMachine {
    id: usize,
    tmpdir: Option<PathBuf>,
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
        manager: &mut MachinesManager,
    ) -> Self {
        let ssh = SshConnection::new(user, host, pkey, password, port);

        let id = manager.generate_id();
        let machine = Self {
            ssh: RefCell::new(ssh),
            tmpdir: None,
            should_remove_tmpdir: true,
            id,
        };

        manager.add_machine(Box::new(machine.clone()));
        machine
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

    fn get_id(&self) -> usize {
        self.id
    }

    fn connect(&mut self) -> Result<(), CrustError> {
        self.ssh.borrow_mut().connect()
    }
}

/// Implementation of temporary directory handling.
impl TemporaryDirectory for RemoteMachine {
    fn can_be_removed(&self) -> bool {
        self.should_remove_tmpdir
    }

    fn get_tmpdir(&self) -> PathBuf {
        self.tmpdir
            .clone()
            .expect("Temporary directory was not created")
    }

    fn tmpdir_exists(&self) -> bool {
        self.tmpdir.is_some()
    }

    fn create_tmpdir(&mut self) -> Result<PathBuf, CrustError> {
        if self.tmpdir_exists() {
            return Ok(self.tmpdir.clone().unwrap());
        }

        let sftp = self.get_session().unwrap().sftp()?;

        let temp_dir_path = PathBuf::from(format!("/tmp/tmp.{}", Uuid::new_v4().as_u128()));
        sftp.mkdir(&temp_dir_path, 0o755)?;

        self.tmpdir = Some(temp_dir_path.clone());
        Ok(temp_dir_path)
    }

    fn create_tmpdir_content(&self, filename: &str) -> Result<PathBuf, CrustError> {
        if !self.tmpdir_exists() {
            return Err(CrustError {
                code: ExitCode::Remote,
                message: "You wanted to create tempfile, but you have not created tempdir!"
                    .to_string(),
            });
        }

        let sftp = self.get_session().unwrap().sftp()?;
        let path = PathBuf::from(self.tmpdir.as_ref().unwrap()).join(filename);
        sftp.create(&path)?;

        Ok(path)
    }

    fn remove_tmpdir(&self) -> Option<Result<(), CrustError>> {
        if self.can_be_removed() && self.tmpdir_exists() {
            let sftp = match self.get_session().unwrap().sftp() {
                Ok(s) => s,
                Err(e) => {
                    return Some(Err(CrustError {
                        code: ExitCode::Remote,
                        message: e.to_string(),
                    }))
                }
            };

            return match sftp.rmdir(self.tmpdir.as_ref().unwrap()) {
                Ok(_) => Some(Ok(())),
                Err(e) => Some(Err(CrustError {
                    code: ExitCode::Remote,
                    message: e.to_string(),
                })),
            };
        }
        None
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

/// Add 'scp' method for RemoteMachine
impl Scp for RemoteMachine {
    fn get_address(&self) -> String {
        self.ssh_address()
    }

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
            id: self.id,
        }
    }
}
