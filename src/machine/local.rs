use std::cell::RefCell;
use std::fs::DirBuilder;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::rc::Rc;

use uuid::Uuid;

use crate::connection::manager::{MachinesManager, MachinesManagerMethods};
use crate::error::{CrustError, ExitCode};
use crate::exec::Exec;
use crate::interfaces::{response::CrustResult, tmpdir::TemporaryDirectory};
use crate::machine::{Machine, MachineID, MachineType};

/// Definition of LocalMachine with private fields.
/// - id: machine id for MachinesManager
/// - tmpdir: possible path to temporary directory
/// - should_remove_tmpdir: determines whether dir
///   should be removed on dropping object
pub struct LocalMachine {
    id: MachineID,
    tmpdir: Option<PathBuf>,
    should_remove_tmpdir: bool,
}

/// Set of unique methods for this LocalMachine structure.
impl LocalMachine {
    /// Creates a new Localmachine. This method should be used only
    /// in one-scope calls (doesn't have any background utils).
    pub fn new() -> Self {
        Self {
            tmpdir: None,
            should_remove_tmpdir: true,
            id: LocalMachine::generate_id(),
        }
    }

    /// Main method to get/create machine in background mode.
    /// Generates ID for local machine and checks if MachinesManager
    /// already stores it. If yes, return reference to stored machine.
    /// Otherwise create a new one, add to manager and return it.
    pub fn get_or_create(manager: &mut MachinesManager) -> Rc<RefCell<Box<dyn Machine>>> {
        match manager.get_machine(&LocalMachine::generate_id()) {
            Some(machine) => machine.clone(),
            None => manager.add_machine(Box::new(LocalMachine::new())),
        }
    }

    /// Private method to generate id for local machine.
    fn generate_id() -> MachineID {
        MachineID::default()
    }
}

/// Default localmachine implementation without connections manager.
impl Default for LocalMachine {
    fn default() -> Self {
        LocalMachine {
            tmpdir: None,
            should_remove_tmpdir: true,
            id: LocalMachine::generate_id(),
        }
    }
}

/// Provides methods from Machine trait to deliver a common interface.
impl Machine for LocalMachine {
    #[inline(always)]
    fn mtype(&self) -> MachineType {
        MachineType::LocalMachine
    }

    fn get_ssh(&self) -> Option<crate::connection::SshConnection> {
        None
    }

    #[inline(always)]
    fn get_session(&self) -> Option<ssh2::Session> {
        None
    }

    fn get_id(&self) -> &MachineID {
        &self.id
    }

    #[inline(always)]
    fn connect(&mut self) -> Result<(), CrustError> {
        Ok(())
    }

    #[inline(always)]
    fn is_connected(&self) -> bool {
        true
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

    fn exec_rt(&self, cmd: &str, merge_pipes: bool) -> Result<CrustResult, CrustError> {
        match merge_pipes {
            true => {
                let child = Command::new("sh")
                    .arg("-c")
                    .arg(&format!("{cmd} 2>&1"))
                    .stdout(Stdio::piped())
                    .spawn()?;

                if let Some(out) = child.stdout {
                    BufReader::new(out)
                        .lines()
                        .map_while(Result::ok)
                        .for_each(|line| println!("{line}"));
                } else {
                    return Err(CrustError {
                        code: ExitCode::Local,
                        message: String::from("STDOUT & STDERR are empty"),
                    });
                }
            }
            false => {
                let child = Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .stderr(Stdio::piped())
                    .spawn()?;

                if let Some(e) = child.stderr {
                    BufReader::new(e)
                        .lines()
                        .map_while(Result::ok)
                        .for_each(|line| log::error!("{line}"));
                } else {
                    return Err(CrustError {
                        code: ExitCode::Local,
                        message: String::from("STDERR is empty"),
                    });
                }
            }
        };

        Ok(CrustResult::default())
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
            id: self.id.clone(),
        }
    }
}

impl std::fmt::Display for LocalMachine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "LocalMachine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localmachine_drop_no_remove_dir() {
        let machine = LocalMachine::new();
        let mut cloned = machine.clone();
        let path = cloned.create_tmpdir().unwrap();

        std::mem::drop(cloned);
        assert!(path.exists());
    }

    #[test]
    fn test_localmachine_drop_success() {
        let mut machine = LocalMachine::new();
        let path = machine.create_tmpdir().unwrap();

        std::mem::drop(machine);
        assert!(!path.exists());
    }

    #[test]
    fn test_exec_localmachine_failed() {
        let machine = LocalMachine::new();
        let result = machine.exec("abc");

        assert!(result.is_ok());
        let res = result.unwrap();

        assert_eq!(res.stdout(), "");
        assert_eq!(res.stderr(), "sh: 1: abc: not found\n");
        assert_eq!(res.retcode(), 127);
    }

    #[test]
    fn test_exec_localmachine_success() {
        let machine = LocalMachine::new();
        let result = machine.exec("echo 'test'");

        assert!(result.is_ok());
        let res = result.unwrap();

        assert_eq!(res.stdout(), "test\n");
        assert_eq!(res.stderr(), "");
        assert_eq!(res.retcode(), 0);
    }

    #[test]
    fn test_clone_localmachine() {
        let machine = LocalMachine::new();

        let cloned = machine.clone();

        assert_eq!(machine.get_id(), cloned.get_id());
        assert!(!cloned.can_be_removed());
    }

    #[test]
    fn test_remove_localmachine_tmpdir() {
        let mut machine = LocalMachine::new();
        let path = machine.create_tmpdir().unwrap();
        let result = machine.remove_tmpdir();

        assert!(result.is_ok());
        assert!(!path.exists());
    }

    #[test]
    fn test_create_content_for_localmachine() {
        let mut machine = LocalMachine::new();
        let _ = machine.create_tmpdir();
        let result = machine.create_tmpdir_content("abc");
        assert!(result.is_ok());

        let path = result.ok().unwrap();
        assert!(path.exists());

        let _ = std::fs::remove_dir_all(path);
    }

    #[test]
    fn test_create_content_for_localmachine_tmpdir_doesnt_exist() {
        let machine = LocalMachine::new();

        let result = machine.create_tmpdir_content("abc");
        assert!(result.is_err());

        let err = result.err().unwrap();
        assert_eq!(err.code, ExitCode::Local);
        assert_eq!(
            err.message,
            "You wanted to create tempfile, but you have not created tempdir!"
        );
    }

    #[test]
    fn test_create_tmpdir_for_localmachine() {
        let mut machine = LocalMachine::new();

        let tmp_dir_result = machine.create_tmpdir();
        assert!(tmp_dir_result.is_ok());
        assert!(machine.tmpdir_exists());

        let tmp_dir_path = tmp_dir_result.ok().unwrap();
        assert!(tmp_dir_path.exists());

        let _ = std::fs::remove_dir_all(tmp_dir_path);
    }

    #[test]
    fn test_create_default_machine() {
        let machine = LocalMachine::default();

        assert!(!machine.tmpdir_exists());
        assert!(machine.can_be_removed());
        assert_eq!(machine.get_id(), &MachineID::default());
    }

    #[test]
    fn test_create_localmachine_without_manager() {
        let machine = LocalMachine::new();

        assert_eq!(machine.tmpdir_exists(), false);
        assert_eq!(machine.get_id(), &MachineID::new(None, None, None));
        assert_eq!(machine.can_be_removed(), true);
        assert_eq!(machine.mtype(), MachineType::LocalMachine);
    }

    #[test]
    fn test_create_localmachine_if_not_present_in_manager_store() {
        let mut manager = MachinesManager::new();
        assert_eq!(manager.size(), 0);

        let machine = LocalMachine::get_or_create(&mut manager);
        assert_eq!(manager.size(), 1);
        assert_eq!(machine.borrow().exec("pwd").unwrap().is_success(), true);
    }

    #[test]
    fn test_get_localmachine_instead_of_creating_a_new() {
        let mut manager = MachinesManager::new();
        assert_eq!(manager.size(), 0);

        LocalMachine::get_or_create(&mut manager);
        assert_eq!(manager.size(), 1);

        LocalMachine::get_or_create(&mut manager);
        assert_eq!(manager.size(), 1);
    }

    #[test]
    fn test_generate_local_id() {
        assert_eq!(
            LocalMachine::generate_id(),
            MachineID::new(None, None, None)
        )
    }
}
