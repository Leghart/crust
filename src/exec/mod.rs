use crate::{error::CrustError, interfaces::response::CrustResult};
pub mod parser;

pub const BUFF_SIZE: usize = 4096;

/// Set of methods required to make an 'execute' command.
/// In case of chosen base, could be used on LocalMachine, RemoteMachine
/// or AbstractMachine.
pub trait Exec {
    /// Execute command on machine. Captures stdout & stderr and
    /// returns them in CrustResult struct.
    fn exec(&self, cmd: &str) -> Result<CrustResult, CrustError>;

    /// Execute command on machine and log stdout & stderr in real time.
    /// For cases where order of logs is important, use `merge_pipes=true` -
    /// both pipes are merged into one (stderr > stdout). Otherwise you will
    /// get stdout as info!, stderr as error!.
    fn exec_rt(&self, cmd: &str, merge_pipes: bool) -> Result<(), CrustError>;
}
