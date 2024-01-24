use crate::error::CrustError;
use crate::error::ExitCode;
use crate::machine::base::Machine;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

pub trait MachinesManagerMethods {
    /// Getter for last machine's id from struct's store.
    fn get_last_id(&self) -> Option<usize>;

    /// Create a unique id for machine. For first invoke,
    /// return 1, otherwise a value is one greater.
    /// TODO!: will be unsafe in case of multi threaded approach
    fn generate_id(&self) -> usize {
        match self.get_last_id() {
            None => 1,
            Some(id) => id + 1,
        }
    }

    /// Adds machine object to internal store (map). If any error related to
    /// adding machine occurred, return Error. Otherwise return ID of new
    /// added element.
    fn add_machine(&mut self, machine: Box<dyn Machine>) -> usize;

    /// Removes requested machine via passed id.
    fn remove_machine(&mut self, id: usize) -> Result<(), CrustError>;

    /// Gets a reference to stored machine.
    /// # Example
    /// ```
    /// let id = 1;
    /// let machine = manager.get_machine(id).unwrap().borrow_mut();
    /// machine.exec("pwd")?
    /// ```
    fn get_machine(&self, id: usize) -> Option<&Rc<RefCell<Box<dyn Machine>>>>;

    /// Reconnect to target machine. If conenction is single, just open
    /// connection again. In case of more complex examples, go through
    /// every proxy and establish connection on each machine if it is broken.
    fn reconnect(&mut self, _: usize) -> Result<(), CrustError> {
        unimplemented!("TODO: will be added after subconnections are handled")
    }
}

pub struct MachinesManager {
    store: HashMap<usize, Rc<RefCell<Box<dyn Machine>>>>,
    //TODO!: in the future add map for related subconnections
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
    fn get_last_id(&self) -> Option<usize> {
        self.store.keys().cloned().max()
    }

    fn add_machine(&mut self, machine: Box<dyn Machine>) -> usize {
        let id = machine.get_id();
        let rc_machine = Rc::new(RefCell::new(machine));
        self.store.insert(id, Rc::clone(&rc_machine));
        id
    }

    fn get_machine(&self, id: usize) -> Option<&Rc<RefCell<Box<dyn Machine>>>> {
        self.store.get(&id)
    }

    fn remove_machine(&mut self, id: usize) -> Result<(), CrustError> {
        if !self.store.contains_key(&id) {
            return Err(CrustError {
                code: ExitCode::Internal,
                message: format!("MachinesManager does not contain Machine<{}>", id),
            });
        }
        self.store.remove(&id);
        Ok(())
    }
}

impl fmt::Display for MachinesManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.store)
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
    fn test_generates_id() {
        let mut manager = MachinesManager::new();

        assert_eq!(manager.generate_id(), 1);
        assert_eq!(manager.generate_id(), 1);
        let _ = LocalMachine::new(&mut manager);
        assert_eq!(manager.generate_id(), 2);
    }

    #[test]
    fn test_get_size_manager() {
        let mut manager = MachinesManager::new();

        assert_eq!(manager.size(), 0);
        let _ = LocalMachine::new(&mut manager);
        assert_eq!(manager.size(), 1);
    }

    #[test]
    fn test_add_machines_to_store() {
        let mut manager = MachinesManager::new();

        let machine1 = Box::new(MockMachine { id: 1 });
        let machine2 = Box::new(MockMachine { id: 2 });

        assert_eq!(manager.add_machine(machine1), 1);
        assert_eq!(manager.size(), 1);

        assert_eq!(manager.add_machine(machine2), 2);
        assert_eq!(manager.size(), 2);
    }

    #[test]
    fn test_remove_machines_from_store() {
        let mut manager = MachinesManager::new();

        let machine1 = Box::new(MockMachine { id: 1 });
        let machine2 = Box::new(MockMachine { id: 2 });

        manager.add_machine(machine1);
        manager.add_machine(machine2);

        manager.remove_machine(1).unwrap();

        assert_eq!(manager.size(), 1);
        assert_eq!(manager.get_last_id(), Some(2));
    }

    #[test]
    fn test_get_machine_from_store() {
        let mut manager = MachinesManager::new();

        let machine1 = Box::new(MockMachine { id: 1 });

        manager.add_machine(machine1);

        let machine = manager.get_machine(1).unwrap().borrow();

        assert_eq!(machine.exec("cmd").unwrap(), "ok");
    }
}
