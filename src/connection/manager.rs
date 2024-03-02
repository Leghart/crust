use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::error::CrustError;
use crate::error::ExitCode;
use crate::machine::DefaultMachineID;
use crate::machine::{Machine, MachineID};

pub trait MachinesManagerMethods {
    /// Adds machine object to internal store (map). If any error related to
    /// adding machine occurred, return Error. Otherwise return ID of new
    /// added element.
    /// # Example
    /// ```
    /// use crate::crust::connection::manager::MachinesManagerMethods;
    /// use crust::connection::manager::MachinesManager;
    /// use crust::machine::local::LocalMachine;
    /// use crust::machine::MachineID;
    ///
    /// let mut manager = MachinesManager::default();
    /// let machine_ref = manager.add_machine(Box::new(LocalMachine::default()));
    /// machine_ref.borrow().exec("pwd").expect("Command failed");
    /// ```    
    fn add_machine(&mut self, machine: Box<dyn Machine>) -> Rc<RefCell<Box<dyn Machine>>>;

    /// Removes requested machine via passed id.
    fn remove_machine(&mut self, id: MachineID) -> Result<(), CrustError>;

    /// Gets a reference to stored machine.
    fn get_machine(&self, id: &MachineID) -> Option<&Rc<RefCell<Box<dyn Machine>>>>;

    // fn get_id_by_alias(&self, alias: &str) -> Option<MachineID>;

    /// Reconnect to target machine. If conenction is single, just open
    /// connection again. In case of more complex examples, go through
    /// every proxy and establish connection on each machine if it is broken.
    fn reconnect(&mut self, _: usize) -> Result<(), CrustError> {
        unimplemented!("TODO: will be added after subconnections are handled")
    }
}

pub struct MachinesManager {
    store: HashMap<MachineID, Rc<RefCell<Box<dyn Machine>>>>,
    //TODO: in the future add map for related subconnections
}

impl MachinesManager {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    /// Gets a current store size.
    pub fn size(&self) -> usize {
        self.store.len()
    }
}

impl MachinesManagerMethods for MachinesManager {
    fn add_machine(&mut self, machine: Box<dyn Machine>) -> Rc<RefCell<Box<dyn Machine>>> {
        let id = machine.get_id().clone();
        let rc_machine = Rc::new(RefCell::new(machine));

        self.store.insert(id.clone(), Rc::clone(&rc_machine));

        log::debug!("Added {:?} to manager", &rc_machine.borrow());
        log::debug!("Manager's store:{} [size:{}]", self, self.store.len());
        rc_machine
    }

    fn get_machine(&self, id: &MachineID) -> Option<&Rc<RefCell<Box<dyn Machine>>>> {
        self.store.get(id)
    }

    fn remove_machine(&mut self, id: MachineID) -> Result<(), CrustError> {
        if !self.store.contains_key(&id) {
            return Err(CrustError {
                code: ExitCode::Internal,
                message: format!("MachinesManager does not contain Machine<{id}>"),
            });
        }
        self.store.remove(&id);
        log::debug!("Removed machine ({id})");
        Ok(())
    }

    // fn get_id_by_alias(&self, alias: &str) -> Option<MachineID> {
    //     None
    // }
}

impl fmt::Display for MachinesManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let entries = self
            .store
            .iter()
            .map(|(k, v)| format!("\n\t{}: {}", k, v.borrow()))
            .collect::<Vec<_>>()
            .join(",\n");
        write!(f, "{{{}\n}}", entries)
    }
}

impl Default for MachinesManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::local::LocalMachine;
    use crate::mocks::machine::MockMachine;

    #[test]
    fn test_get_size_manager() {
        let mut manager = MachinesManager::new();

        assert_eq!(manager.size(), 0);
        LocalMachine::get_or_create(&mut manager);
        assert_eq!(manager.size(), 1);
    }

    #[test]
    fn test_add_machines_to_store() {
        let mut manager = MachinesManager::new();

        let machine1 = Box::new(MockMachine {
            id: MachineID::new(Some(String::from("a")), Some(String::from("b")), Some(1)),
            tmpdir: None,
        });
        let machine2 = Box::new(MockMachine {
            id: MachineID::default(),
            tmpdir: None,
        });

        manager.add_machine(machine1);
        assert_eq!(manager.size(), 1);

        manager.add_machine(machine2);
        assert_eq!(manager.size(), 2);
    }

    #[test]
    fn test_remove_machines_from_store() {
        let mut manager = MachinesManager::new();

        let machine1 = Box::new(MockMachine {
            id: MachineID::new(Some(String::from("a")), Some(String::from("b")), Some(1)),
            tmpdir: None,
        });

        let machine2 = Box::new(MockMachine {
            id: MachineID::default(),
            tmpdir: None,
        });

        manager.add_machine(machine1);
        manager.add_machine(machine2);

        manager.remove_machine(MachineID::default()).unwrap();

        assert_eq!(manager.size(), 1);
    }

    #[test]
    fn test_get_machine_from_store() {
        let mut manager = MachinesManager::new();

        let machine1 = Box::new(MockMachine {
            id: MachineID::default(),
            tmpdir: None,
        });

        manager.add_machine(machine1);

        let machine = manager.get_machine(&MachineID::default()).unwrap().borrow();

        assert_eq!(machine.exec("cmd").unwrap().is_success(), true);
    }
}
