use ssh2::Session;

use crate::exec::Exec;
use crate::interfaces::tmpdir::TemporaryDirectory;
use crate::tscp::Tscp;

/// Set of common methods for local and remote machines. It could
/// be seen as abstract class, which must be overriden by childs.
pub trait Machine: TemporaryDirectory + Exec + Tscp {
    /// Defines a type of machine.
    /// Possible choices are: LocalMachine, RemoteMachine, AbstractMachine
    fn mtype(&self) -> MachineType;

    /// Get a name of structure. Could be changed with `Display` trait but
    /// in case of localmachine - empty string could be a bit suspicious.
    fn ssh_address(&self) -> String;

    /// Getter for possible session object (only machines where SSH connection
    /// is required). In the case of LocalMachine, it immediately returns None.
    fn get_session(&self) -> Option<Session>;
}

/// Enum which allow to recognize dynamicly-created objects (as
/// common interface must be used as references in pointers [Box]).
/// This could be treated as argument for `isinstance`.
pub enum MachineType {
    AbstractMachine,
    LocalMachine,
    RemoteMachine,
}
