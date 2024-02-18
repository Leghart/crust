use super::base::{Machine, MachineType};
use std::fs::DirBuilder;
use std::path::PathBuf;
use std::process::Command;
use uuid::Uuid;

use crate::connection::manager::{MachinesManager, MachinesManagerMethods};
use crate::interfaces::{response::CrustResult, tmpdir::TemporaryDirectory};

use crate::error::{CrustError, ExitCode};
use crate::exec::Exec;
use crate::scp::Scp;

/// Definition of LocalMachine with private fields.
/// - id: machine id for MachinesManager
/// - tmpdir: possible path to temporary directory
/// - should_remove_tmpdir: determines whether dir
///   should be removed on dropping object
pub struct LocalMachine {
    id: Option<usize>,
    tmpdir: Option<PathBuf>,
    should_remove_tmpdir: bool,
}

/// Set of unique methods for this LocalMachine structure.
impl LocalMachine {
    pub fn new(manager: &mut MachinesManager) -> Self {
        let machine = Self {
            tmpdir: None,
            should_remove_tmpdir: true,
            id: Some(manager.generate_id()),
        };

        manager.add_machine(Box::new(machine.clone()));

        machine
    }
}

/// Default localmachine implementation without connections manager.
impl Default for LocalMachine {
    fn default() -> Self {
        LocalMachine {
            id: None,
            tmpdir: None,
            should_remove_tmpdir: true,
        }
    }
}

/// Provides methods from Machine trait to deliver a common interface.
impl Machine for LocalMachine {
    #[inline(always)]
    fn mtype(&self) -> MachineType {
        MachineType::LocalMachine
    }

    #[inline(always)]
    fn get_session(&self) -> Option<ssh2::Session> {
        None
    }

    fn get_id(&self) -> Option<usize> {
        self.id
    }

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

    fn get_tmpdir(&self) -> &PathBuf {
        self.tmpdir
            .as_ref()
            .expect("Temporary directory was not created")
    }

    fn create_tmpdir(&mut self) -> Result<PathBuf, CrustError> {
        if self.tmpdir_exists() {
            log::warn!("Temp dir for {} already exists", self);
            return Ok(self.tmpdir.clone().unwrap());
        }

        let temp_dir_path = PathBuf::from(format!("/tmp/tmp.{}", Uuid::new_v4().as_u128()));
        DirBuilder::new().create(&temp_dir_path)?;

        self.tmpdir = Some(PathBuf::from(&temp_dir_path));
        Ok(temp_dir_path)
    }

    fn create_tmpdir_content(&self, filename: &str) -> Result<PathBuf, CrustError> {
        if !self.tmpdir_exists() {
            return Err(CrustError {
                code: ExitCode::Local,
                message: "You wanted to create tempfile, but you have not created tempdir!"
                    .to_string(),
            });
        }
        let path = PathBuf::from(self.tmpdir.as_ref().unwrap()).join(filename);
        std::fs::File::create(&path)?;
        Ok(path)
    }

    fn remove_tmpdir(&self) -> Result<(), CrustError> {
        std::fs::remove_dir_all(self.tmpdir.as_ref().unwrap())?;
        Ok(())
    }
}

/// Add `execute` method for LocalMachine
impl Exec for LocalMachine {
    fn exec(&self, cmd: &str) -> Result<CrustResult, CrustError> {
        let result = Command::new("sh").arg("-c").arg(cmd).output()?;

        Ok(CrustResult::new(
            &String::from_utf8(result.stdout)?,
            &String::from_utf8(result.stderr)?,
            result.status.code().unwrap_or(1),
        ))
    }
}

/// Add 'scp' method for LocalMachine
impl Scp for LocalMachine {
    fn get_machine(&self) -> MachineType {
        self.mtype()
    }
}

/// Destructur implemtation for cleanup temporary directory when
/// struct leaves scope.
impl Drop for LocalMachine {
    fn drop(&mut self) {
        if self.tmpdir_exists() && self.can_be_removed() {
            let _ = self.remove_tmpdir();
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

impl std::fmt::Display for LocalMachine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let id_str = match self.id {
            Some(i) => i.to_string(),
            None => String::from("-"),
        };

        write!(f, "LocalMachine[{id_str}]")
    }
}
