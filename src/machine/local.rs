use super::base::{Machine, MachineType};

use std::process::Command;

use crate::connection::manager::{MachinesManager, MachinesManagerMethods};
use crate::interfaces::tmpdir::TemporaryDirectory;

use crate::error::{CrustError, ExitCode};
use crate::exec::Exec;
use crate::scp::Scp;

/// Definition of LocalMachine with private fields.
/// - id: machine id for MachinesManager
/// - tmpdir: possible path to temporary directory
/// - should_remove_tmpdir: determines whether dir
///   should be removed on dropping object
pub struct LocalMachine {
    id: usize,
    tmpdir: Option<String>,
    should_remove_tmpdir: bool,
}

/// Set of unique methods for this LocalMachine structure.
impl LocalMachine {
    pub fn new(manager: &mut MachinesManager) -> Self {
        let id = manager.generate_id();
        let machine = Self {
            tmpdir: None,
            should_remove_tmpdir: true,
            id,
        };

        manager.add_machine(Box::new(machine.clone()));

        machine
    }
}

/// Provides methods from Machine trait to deliver a common interface.
impl Machine for LocalMachine {
    #[inline(always)]
    fn mtype(&self) -> MachineType {
        MachineType::LocalMachine
    }

    #[inline(always)]
    fn ssh_address(&self) -> String {
        "".to_string()
    }

    #[inline(always)]
    fn get_session(&self) -> Option<ssh2::Session> {
        None
    }

    fn get_id(&self) -> usize {
        self.id
    }

    ///TODO!: ugly
    #[inline(always)]
    fn connect(&mut self) -> Result<(), CrustError> {
        Ok(())
    }
}

/// Implementation of temporary directory handling.
impl TemporaryDirectory for LocalMachine {
    fn can_be_removed(&self) -> bool {
        self.should_remove_tmpdir
    }

    fn tmpdir_exists(&self) -> bool {
        self.tmpdir.is_some()
    }

    fn get_tmpdir(&self) -> String {
        self.tmpdir
            .clone()
            .expect("Temporary directory was not created")
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

/// Add `execute` method for LocalMachine
impl Exec for LocalMachine {
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
}

/// Add 'scp' method for LocalMachine
impl Scp for LocalMachine {
    fn get_address(&self) -> String {
        self.ssh_address()
    }

    fn get_machine(&self) -> MachineType {
        self.mtype()
    }
}

/// Destructur implemtation for cleanup temporary directory when
/// struct leaves scope.
impl Drop for LocalMachine {
    fn drop(&mut self) {
        if self.tmpdir_exists() {
            self.remove_tmpdir();
        }
    }
}

/// Custom Clone implementation that guarantees that
/// copies of the object will not delete the directory.
impl Clone for LocalMachine {
    fn clone(&self) -> Self {
        LocalMachine {
            tmpdir: self.tmpdir.clone(),
            should_remove_tmpdir: false,
            id: self.id,
        }
    }
}
