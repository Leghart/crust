use std::cell::RefCell;
use std::path::PathBuf;

use super::base::{Machine, MachineType};

use crate::connection::manager::{MachinesManager, MachinesManagerMethods};
use crate::connection::{SshConnection, SSH};
use crate::error::CrustError;
use crate::exec::Exec;
use crate::interfaces::tmpdir::TemporaryDirectory;

/// Definition of RemoteMachine with private fields.
/// - id: machine id for MachinesManager
/// - tmpdir: possible path to temporary directory
/// - should_remove_tmpdir: determines whether dir
///   should be removed on dropping object
/// - ssh: reference to `SshConnection` object which
///   provides access to remote servers.
pub struct RemoteMachine {
    id: usize,
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

    fn get_id(&self) -> usize {
        self.id
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
