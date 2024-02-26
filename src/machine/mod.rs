use std::collections::hash_map::DefaultHasher;
use std::fmt::Display;
use std::hash::{Hash, Hasher};

use core::fmt::Debug;
use ssh2::Session;

pub mod local;
pub mod remote;

use crate::error::CrustError;
use crate::exec::Exec;
use crate::interfaces::tmpdir::TemporaryDirectory;
use crate::scp::Scp;

/// Set of common methods for local and remote machines. It could
/// be seen as abstract class, which must be overriden by childs.
pub trait Machine: TemporaryDirectory + Exec + Scp + Display {
    /// Defines a type of machine.
    /// Possible choices are: LocalMachine, RemoteMachine, AbstractMachine
    fn mtype(&self) -> MachineType;

    /// Getter for possible session object (only machines where SSH connection
    /// is required). In the case of LocalMachine, it immediately returns None.
    fn get_session(&self) -> Option<Session>;

    /// Gets a private ID value.
    fn get_id(&self) -> &MachineID;

    /// Required to maintain a common interface.
    fn connect(&mut self) -> Result<(), CrustError>;
}

/// Hashable struct which represents a machine
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Default)]
pub struct MachineID {
    user: Option<String>,
    host: Option<String>,
    port: Option<u16>,
}

impl MachineID {
    pub fn new(user: Option<String>, host: Option<String>, port: Option<u16>) -> Self {
        match (user.is_some(), host.is_some(), port.is_some()) {
            (true, true, true) | (false, false, false) => Self { user, host, port },
            _ => panic!("To generate LocalMachine ID, all values must be None. For RemoteMachine all values must be provided."),
        }
    }
}

impl Display for MachineID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let str_id: String;
        let mut hasher = DefaultHasher::new();
        self.user.hash(&mut hasher);
        self.host.hash(&mut hasher);
        self.port.hash(&mut hasher);
        let str_id = hasher.finish();
        write!(f, "MachineID<{str_id}>")
    }
}

/// Enum which allow to recognize dynamicly-created objects (as
/// common interface must be used as references in pointers [Box]).
/// This could be treated as argument for `isinstance`.
#[derive(Debug, PartialEq)]
pub enum MachineType {
    AbstractMachine,
    LocalMachine,
    RemoteMachine,
}

impl Debug for dyn Machine {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Machine<{}:{:?}>", self.get_id(), self.mtype())
    }
}
