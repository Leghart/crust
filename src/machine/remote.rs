use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use uuid::Uuid;

use crate::connection::manager::{MachinesManager, MachinesManagerMethods};
use crate::connection::{SshConnection, SSH};
use crate::error::{CrustError, ExitCode};
use crate::exec::Exec;
use crate::interfaces::response::CrustResult;
use crate::interfaces::tmpdir::TemporaryDirectory;
use crate::machine::{Machine, MachineID, MachineType};
use crate::scp::Scp;

/// Definition of RemoteMachine with private fields.
/// - id: machine id for MachinesManager
/// - tmpdir: possible path to temporary directory
/// - should_remove_tmpdir: determines whether dir
///   should be removed on dropping object
/// - ssh: reference to `SshConnection` object which
///   provides access to remote servers.
pub struct RemoteMachine {
    id: MachineID,
    tmpdir: Option<PathBuf>,
    should_remove_tmpdir: bool,
    ssh: RefCell<SshConnection>,
}

/// Set of unique methods for this RemoteMachine structure.
impl RemoteMachine {
    /// Creates a new RemoteMachine. This method should be used only
    /// in one-scope calls (doesn't have any background utils).    
    pub fn new(
        user: &str,
        host: &str,
        password: Option<String>,
        pkey: Option<PathBuf>,
        port: u16,
    ) -> Self {
        let ssh = SshConnection::new(user, host, pkey, password, port);

        Self {
            ssh: RefCell::new(ssh),
            tmpdir: None,
            should_remove_tmpdir: true,
            id: RemoteMachine::generate_default_id(user, host, port),
        }
    }

    /// Main method to get/create machine in background mode.
    /// Generates ID for remote machine from passed connection arguments(
    /// if 'alias parameter is None - otherwise ID is this alias) and
    /// checks if MachinesManager already stores it. If yes, return
    /// reference to stored machine. Otherwise create a new one, add to
    /// manager and return it.
    pub fn get_or_create(
        user: String,
        host: String,
        password: Option<String>,
        pkey: Option<PathBuf>,
        port: u16,
        alias: Option<String>,
        manager: &mut MachinesManager,
    ) -> Rc<RefCell<Box<dyn Machine>>> {
        let id = match alias {
            Some(_alias) => RemoteMachine::generate_custom_id(&_alias),
            None => RemoteMachine::generate_default_id(&user, &host, port),
        };

        match manager.get_machine(&id) {
            Some(machine) => machine.clone(),
            None => {
                let ssh = SshConnection::new(&user, &host, pkey, password, port);
                let machine = Self {
                    ssh: RefCell::new(ssh),
                    tmpdir: None,
                    should_remove_tmpdir: true,
                    id,
                };
                manager.add_machine(Box::new(machine))
            }
        }
    }

    /// Tries to get a machine from manager by machine alias.
    /// In case when passed alias is not registered - return None, otherwise
    /// returns reference to machine from manager.
    pub fn get(
        alias: &str,
        manager: &mut MachinesManager,
    ) -> Option<Rc<RefCell<Box<dyn Machine>>>> {
        let id = RemoteMachine::generate_custom_id(alias);
        manager.get_machine(&id).cloned()
    }

    /// Getter for ssh config.
    pub fn get_ssh(&self) -> &RefCell<SshConnection> {
        &self.ssh
    }

    /// Private method to generate id for remote machine.
    fn generate_default_id(user: &str, host: &str, port: u16) -> MachineID {
        MachineID::Default(
            Some(String::from(user)),
            Some(String::from(host)),
            Some(port),
        )
    }

    /// Private method to generate id for remote machine.
    fn generate_custom_id(alias: &str) -> MachineID {
        MachineID::Custom(alias.to_string())
    }
}

/// Provides methods from Machine trait to deliver a common interface.
impl Machine for RemoteMachine {
    #[inline(always)]
    fn mtype(&self) -> MachineType {
        MachineType::RemoteMachine
    }

    fn get_session(&self) -> Option<ssh2::Session> {
        Some(self.ssh.borrow().session().clone())
    }

    fn get_id(&self) -> &MachineID {
        &self.id
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

    fn get_tmpdir(&self) -> &PathBuf {
        self.tmpdir
            .as_ref()
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

    fn remove_tmpdir(&self) -> Result<(), CrustError> {
        //TODO: Workaround to remove direcotry with content
        self.exec(&format!("rm -rf {}", self.get_tmpdir().display()))?;
        Ok(())
    }
}

/// Add `execute` method for RemoteMachine
impl Exec for RemoteMachine {
    fn exec(&self, cmd: &str) -> Result<CrustResult, CrustError> {
        if !self.ssh.borrow().is_connected() {
            self.ssh.borrow_mut().connect()?;
        }
        self.ssh.borrow().execute(cmd)
    }

    fn exec_rt(&self, cmd: &str, merge_pipes: bool) -> Result<CrustResult, CrustError> {
        if !self.ssh.borrow().is_connected() {
            self.ssh.borrow_mut().connect()?;
        }
        self.ssh.borrow().execute_rt(cmd, merge_pipes)
    }
}

/// Add 'scp' method for RemoteMachine
impl Scp for RemoteMachine {
    fn get_machine(&self) -> MachineType {
        self.mtype()
    }
}

/// Destructur implemtation for cleanup temporary directory when
/// struct leaves scope.
impl Drop for RemoteMachine {
    fn drop(&mut self) {
        if self.tmpdir_exists() && self.can_be_removed() {
            let _ = self.remove_tmpdir();
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
            id: self.id.clone(),
        }
    }
}

impl std::fmt::Display for RemoteMachine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "RemoteMachine<{}>", self.ssh.borrow())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use test_utils::{exec_on_remote, exists_on_remote};

    fn connect_args() -> (String, String, Option<String>, Option<PathBuf>, u16) {
        (
            String::from("test_user"),
            String::from("10.10.10.10"),
            Some(String::from("1234")),
            None,
            22,
        )
    }

    #[serial]
    #[test]
    fn test_remotemachine_drop_no_remove_dir() {
        let (user, host, pass, pkey, port) = connect_args();
        let mut machine = RemoteMachine::new(&user, &host, pass, pkey, port);
        let r = machine.connect();
        assert!(r.is_ok());

        let mut cloned = machine.clone();
        let path = cloned.create_tmpdir().unwrap();

        std::mem::drop(cloned);
        assert!(exists_on_remote(path.clone(), true));

        exec_on_remote(&format!("rm -rf {}", path.as_path().to_str().unwrap()));
    }

    #[serial]
    #[test]
    fn test_remotemachine_drop_success() {
        let (user, host, pass, pkey, port) = connect_args();
        let mut machine = RemoteMachine::new(&user, &host, pass, pkey, port);
        let r = machine.connect();
        assert!(r.is_ok());

        let path = machine.create_tmpdir().unwrap();

        std::mem::drop(machine);
        assert!(!exists_on_remote(path, true));
    }

    #[serial]
    #[test]
    fn test_exec_remotemachine_failed() {
        let (user, host, pass, pkey, port) = connect_args();
        let mut machine = RemoteMachine::new(&user, &host, pass, pkey, port);
        let r = machine.connect();
        assert!(r.is_ok());

        let result = machine.exec("abc");

        assert!(result.is_ok());
        let res = result.unwrap();

        assert_eq!(res.stdout(), "");
        assert_eq!(res.stderr(), "bash: abc: command not found\n");
        assert_eq!(res.retcode(), 0); //TODO?: retcode is channel status -> not the same
    }

    #[serial]
    #[test]
    fn test_exec_remotemachine_success() {
        let (user, host, pass, pkey, port) = connect_args();
        let mut machine = RemoteMachine::new(&user, &host, pass, pkey, port);
        let r = machine.connect();
        assert!(r.is_ok());

        let result = machine.exec("echo 'test'");

        assert!(result.is_ok());
        let res = result.unwrap();

        assert_eq!(res.stdout(), "test\n");
        assert_eq!(res.stderr(), "");
        assert_eq!(res.retcode(), 0);
    }

    #[serial]
    #[test]
    fn test_clone_remotemachine() {
        let (user, host, pass, pkey, port) = connect_args();
        let machine = RemoteMachine::new(&user, &host, pass, pkey, port);

        let cloned = machine.clone();

        assert_eq!(machine.get_id(), cloned.get_id());
        assert!(!cloned.can_be_removed());
        let ssh_original = machine.get_ssh().borrow();
        let ssh_cloned = cloned.get_ssh().borrow();
        assert_eq!(ssh_original.to_string(), ssh_cloned.to_string())
    }

    // #[serial]
    // #[test]
    // fn test_remove_remotemachine_tmpdir() {
    //     let (user, host, pass, pkey, port) = connect_args();
    //     let mut machine = RemoteMachine::new(&user, &host, pass, pkey, port);
    //     let r = machine.connect();
    //     assert!(r.is_ok());

    //     let path = machine.create_tmpdir().unwrap();
    //     let result = machine.remove_tmpdir();

    //     assert!(result.is_ok());
    //     assert_eq!(
    //         "",
    //         exec_on_remote(&format!(
    //             "find /tmp -name {}",
    //             path.as_path().to_str().unwrap()
    //         ))
    //     );
    // }

    #[serial]
    #[test]
    fn test_create_content_for_remotemachine() {
        let (user, host, pass, pkey, port) = connect_args();
        let mut machine = RemoteMachine::new(&user, &host, pass, pkey, port);
        let r = machine.connect();
        assert!(r.is_ok());

        let _ = machine.create_tmpdir();
        let result = machine.create_tmpdir_content("abc");
        assert!(result.is_ok());

        let path = result.ok().unwrap();
        assert!(exists_on_remote(path, false));
    }

    #[serial]
    #[test]
    fn test_create_content_for_remotemachine_tmpdir_doesnt_exist() {
        let (user, host, pass, pkey, port) = connect_args();
        let machine = RemoteMachine::new(&user, &host, pass, pkey, port);

        let result = machine.create_tmpdir_content("abc");
        assert!(result.is_err());

        let err = result.err().unwrap();
        assert_eq!(err.code, ExitCode::Remote);
        assert_eq!(
            err.message,
            "You wanted to create tempfile, but you have not created tempdir!"
        );
    }

    #[serial]
    #[test]
    fn test_create_tmpdir_for_remotemachine() {
        let (user, host, pass, pkey, port) = connect_args();
        let mut machine = RemoteMachine::new(&user, &host, pass, pkey, port);
        let r = machine.connect();
        assert!(r.is_ok());

        let tmp_dir_result = machine.create_tmpdir();
        assert!(tmp_dir_result.is_ok());
        assert!(machine.tmpdir_exists());

        let tmp_dir_path = tmp_dir_result.ok().unwrap();
        assert!(exists_on_remote(tmp_dir_path, true));
    }

    #[serial]
    #[test]
    fn test_create_remotemachine_without_manager() {
        let (user, host, pass, pkey, port) = connect_args();
        let machine = RemoteMachine::new(&user, &host, pass, pkey, port);

        assert_eq!(machine.tmpdir_exists(), false);
        assert_eq!(
            machine.get_id(),
            &MachineID::new(Some(user), Some(host), Some(port))
        );
        assert_eq!(machine.can_be_removed(), true);
        assert_eq!(machine.mtype(), MachineType::RemoteMachine);
    }

    #[serial]
    #[test]
    fn test_create_remotemachine_if_not_present_in_manager_store() {
        let mut manager = MachinesManager::new();
        assert_eq!(manager.size(), 0);

        let (user, host, pass, pkey, port) = connect_args();
        let machine =
            RemoteMachine::get_or_create(user, host, pass, pkey, port, None, &mut manager);

        assert_eq!(manager.size(), 1);
        assert_eq!(machine.borrow().exec("pwd").unwrap().is_success(), true);
    }

    #[serial]
    #[test]
    fn test_get_remotemachine_instead_of_creating_a_new() {
        let mut manager = MachinesManager::new();
        assert_eq!(manager.size(), 0);

        RemoteMachine::get_or_create(
            String::from("a"),
            String::from("b"),
            Some(String::from("p")),
            None,
            22,
            None,
            &mut manager,
        );
        assert_eq!(manager.size(), 1);

        RemoteMachine::get_or_create(
            String::from("a"),
            String::from("b"),
            Some(String::from("p")),
            None,
            22,
            None,
            &mut manager,
        );
        assert_eq!(manager.size(), 1);
    }

    #[serial]
    #[test]
    fn test_generate_remote_id() {
        assert_eq!(
            RemoteMachine::generate_default_id("a", "b", 1),
            MachineID::new(Some(String::from("a")), Some(String::from("b")), Some(1))
        )
    }
}
