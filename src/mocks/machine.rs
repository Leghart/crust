use std::path::PathBuf;

use crate::error::CrustError;
use crate::interfaces::response::CrustResult;
use crate::machine::{MachineID, MachineType};
use crate::{exec::Exec, interfaces::tmpdir::TemporaryDirectory, machine::Machine};

pub struct MockMachine {
    pub id: MachineID,
    pub tmpdir: Option<PathBuf>,
}

impl Machine for MockMachine {
    fn get_id(&self) -> &MachineID {
        &self.id
    }
    fn get_ssh(&self) -> Option<crate::connection::SshConnection> {
        None
    }

    fn get_session(&self) -> Option<ssh2::Session> {
        None
    }

    fn mtype(&self) -> MachineType {
        MachineType::LocalMachine
    }

    fn connect(&mut self) -> Result<(), CrustError> {
        Ok(())
    }

    fn is_connected(&self) -> bool {
        true
    }
}
impl Exec for MockMachine {
    fn exec(&self, _: &str) -> Result<CrustResult, CrustError> {
        Ok(CrustResult::default())
    }

    fn exec_rt(&self, _cmd: &str, _merge_pipes: bool) -> Result<CrustResult, CrustError> {
        Ok(CrustResult::default())
    }
}

impl TemporaryDirectory for MockMachine {
    fn can_be_removed(&self) -> bool {
        true
    }

    fn tmpdir_exists(&self) -> bool {
        true
    }

    fn get_tmpdir(&self) -> &PathBuf {
        self.tmpdir.as_ref().unwrap()
    }

    fn create_tmpdir(&mut self) -> Result<PathBuf, CrustError> {
        Ok(self.get_tmpdir().clone())
    }

    fn remove_tmpdir(&self) -> Result<(), CrustError> {
        Ok(())
    }

    fn create_tmpdir_content(&self, _filename: &str) -> Result<PathBuf, CrustError> {
        Ok(PathBuf::from(self.get_tmpdir()).join("file"))
    }
}

impl std::fmt::Display for MockMachine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "MockMachine")
    }
}
