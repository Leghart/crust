use crate::error::CrustError;
use crate::machine::base::MachineType;
use crate::{exec::Exec, interfaces::tmpdir::TemporaryDirectory, machine::base::Machine, scp::Scp};

pub struct MockMachine {
    pub id: usize,
}
impl Machine for MockMachine {
    fn get_id(&self) -> usize {
        self.id
    }
    fn get_session(&self) -> Option<ssh2::Session> {
        None
    }
    fn mtype(&self) -> MachineType {
        MachineType::LocalMachine
    }
    fn ssh_address(&self) -> String {
        "".to_string()
    }
    fn connect(&mut self) -> Result<(), CrustError> {
        Ok(())
    }
}
impl Exec for MockMachine {
    fn exec(&self, _: &str) -> Result<String, CrustError> {
        Ok("ok".to_string())
    }
}
impl Scp for MockMachine {
    fn get_address(&self) -> String {
        self.ssh_address()
    }
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
    fn get_tmpdir(&self) -> String {
        "tmpdir".to_string()
    }
    fn create_tmpdir(&mut self) {}

    fn remove_tmpdir(&self) {}
}
