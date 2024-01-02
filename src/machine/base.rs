use std::path::PathBuf;

use crate::error::CrustError;

/// Set of common methods for local and remote machines. It could
/// be seen as abstract class, which must be overriden by childs.
pub trait Machine {
    /// Defines a type of machine.
    /// Possible choices are: LocalMachine, RemoteMachine
    fn mtype(&self) -> MachineType;

    /// Allow to execute passed command on machine. Each machine
    /// has another way of executing (localmachine uses a os, remote
    /// uses SSH tunnel).  
    fn exec(&self, cmd: &str) -> Result<String, CrustError>;

    /// Merges a chunks of source into destination on `dst` path.
    fn merge(&self, dst: &str) -> Result<(), CrustError>;

    /// Splits source data into chunks with passed size. Every chunk
    /// will be saved on temporaty directory, created per each structure.
    fn split(&self, size: u64, data: &str) -> Result<Vec<PathBuf>, CrustError>;

    /// Get a name of structure. Could be changed with `Display` trait but
    /// in case of localmachine - empty string could be a bit suspicious.
    fn ssh_address(&self) -> String;

    /// Creates a temporary director required in by copy methods for
    /// storing chunk data.
    fn create_tmpdir(&mut self) -> String;

    /// Property to get a private `tmpdir` path.
    fn get_tmpdir(&self) -> String;
}

/// Enum which allow to recognize dynamicly-created objects (as
/// common interface must be used as references in pointers [Box]).
/// This could be treated as argument for `isinstance`.
pub enum MachineType {
    AbstractMachine,
    LocalMachine,
    RemoteMachine,
}
