use std::collections::hash_map::DefaultHasher;
use std::fmt::Display;
use std::hash::{Hash, Hasher};

use core::fmt::Debug;
use ssh2::Session;

pub mod local;
pub mod remote;

use crate::connection::SshConnection;
use crate::error::CrustError;
use crate::exec::Exec;
use crate::interfaces::tmpdir::TemporaryDirectory;

/// Set of common methods for local and remote machines. It could
/// be seen as abstract class, which must be overriden by childs.
pub trait Machine: TemporaryDirectory + Exec + Display {
    /// Defines a type of machine.
    /// Possible choices are: LocalMachine, RemoteMachine, AbstractMachine
    fn mtype(&self) -> MachineType;

    /// Getter for possible session object (only machines where SSH connection
    /// is required). In the case of LocalMachine, it immediately returns None.
    fn get_session(&self) -> Option<Session>;

    /// Gets a private ID value.
    fn get_id(&self) -> &MachineID;

    /// Gets an existing SSH connetion (if exists)
    fn get_ssh(&self) -> Option<SshConnection>;

    /// Required to maintain a common interface.
    fn connect(&mut self) -> Result<(), CrustError>;

    /// Checks whether machine is connected (connection is alive).
    fn is_connected(&self) -> bool;
}

/// Hashable enum represents a machine ID. There are two options to make
/// an ID:
/// - [defualt] auto-create by arguments represeting machine - user, host and port
/// - custom by passed alias to machine.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub enum MachineID {
    Default(Option<String>, Option<String>, Option<u16>),
    Custom(String),
}

impl Default for MachineID {
    fn default() -> Self {
        MachineID::Default(None, None, None)
    }
}

impl MachineID {
    pub fn new(user: Option<String>, host: Option<String>, port: Option<u16>) -> Self {
        match (user.is_some(), host.is_some(), port.is_some()) {
            (true, true, true) | (false, false, false) => MachineID::Default(user, host, port),
            _ => panic!("To generate LocalMachine ID, all values must be None. For RemoteMachine all values must be provided."),
        }
    }
}

impl Display for MachineID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_id: String = match self {
            MachineID::Default(user, host, port) => {
                let mut hasher = DefaultHasher::new();
                user.hash(&mut hasher);
                host.hash(&mut hasher);
                port.hash(&mut hasher);
                hasher.finish().to_string()
            }
            MachineID::Custom(s) => s.to_string(),
        };

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
