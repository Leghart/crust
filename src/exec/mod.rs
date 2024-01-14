use crate::error::CrustError;
pub mod parser;

/// Set of methods required to make an 'execute' command.
/// In case of chosen base, could be used on LocalMachine, RemoteMachine
/// or AbstractMachine.
pub trait Exec {
    fn exec(&self, cmd: &str) -> Result<String, CrustError>;
}
