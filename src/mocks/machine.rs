use crate::error::CrustError;
use crate::{exec::Exec, interfaces::tmpdir::TemporaryDirectory, machine::base::Machine};

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
    fn mtype(&self) -> crate::machine::base::MachineType {
        crate::machine::base::MachineType::LocalMachine
    }
    fn ssh_address(&self) -> String {
        "".to_string()
    }
}
impl Exec for MockMachine {
    fn exec(&self, _: &str) -> Result<String, CrustError> {
        Ok("ok".to_string())
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
