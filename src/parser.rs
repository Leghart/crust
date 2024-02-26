use crate::exec::parser::ExecArgs;
use crate::interfaces::parser::Validation;
use crate::scp::parser::ScpArgs;
use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;

#[derive(Parser, Debug, Clone)]
#[clap(author = "@Leghart @WiktorNowak", version = "1.0.0", about)]
/// Main parser
pub struct AppArgs {
    #[clap(subcommand)]
    operation: Operation,

    #[clap(flatten)]
    pub verbose: Verbosity,

    #[clap(short, long, default_value = "false")]
    pub background: bool,
}

impl AppArgs {
    pub fn get_operation(&self) -> &Operation {
        &self.operation
    }
}

impl Validation for AppArgs {
    fn validate(&mut self) -> Result<(), crate::error::CrustError> {
        self.operation.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum Operation {
    /// Execute command on machine.
    Exec(ExecArgs),

    /// Copies data between two machines
    Scp(ScpArgs),
}

impl Validation for Operation {
    fn validate(&mut self) -> Result<(), crate::error::CrustError> {
        match self {
            Operation::Exec(args) => args.validate()?,
            Operation::Scp(args) => args.validate()?,
        }
        Ok(())
    }
}
