use ssh2::Session;

use crate::error::CrustError;
use crate::exec::Exec;
use crate::interfaces::tmpdir::TemporaryDirectory;
use crate::scp::Scp;

/// Set of common methods for local and remote machines. It could
/// be seen as abstract class, which must be overriden by childs.
pub trait Machine: TemporaryDirectory + Exec + Scp {
    /// Defines a type of machine.
    /// Possible choices are: LocalMachine, RemoteMachine, AbstractMachine
    fn mtype(&self) -> MachineType;

    /// Get a name of structure. Could be changed with `Display` trait but
    /// in case of localmachine - empty string could be a bit suspicious.
    fn ssh_address(&self) -> String;

    /// Getter for possible session object (only machines where SSH connection
    /// is required). In the case of LocalMachine, it immediately returns None.
    fn get_session(&self) -> Option<Session>;

    /// Gets a private ID value.
    fn get_id(&self) -> usize;

    /// Required to maintain a common interface.
    fn connect(&mut self) -> Result<(), CrustError>;
}

/// Enum which allow to recognize dynamicly-created objects (as
/// common interface must be used as references in pointers [Box]).
/// This could be treated as argument for `isinstance`.
#[derive(Debug)]
pub enum MachineType {
    AbstractMachine,
    LocalMachine,
    RemoteMachine,
}

use core::fmt::Debug;
impl Debug for dyn Machine {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Machine<{:?}>", self.get_id())
    }
}
