use std::path::PathBuf;

use crate::error::CrustError;
use crate::interfaces::response::CrustResult;
use crate::machine::base::MachineType;
use crate::{exec::Exec, interfaces::tmpdir::TemporaryDirectory, machine::base::Machine, scp::Scp};

pub struct MockMachine {
    pub id: Option<usize>,
    pub tmpdir: Option<PathBuf>,
}

impl Machine for MockMachine {
    fn get_id(&self) -> Option<usize> {
        self.id
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
}
impl Exec for MockMachine {
    fn exec(&self, _: &str) -> Result<CrustResult, CrustError> {
        Ok(CrustResult::default())
    }
}

impl Scp for MockMachine {
    fn get_machine(&self) -> MachineType {
        self.mtype()
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
