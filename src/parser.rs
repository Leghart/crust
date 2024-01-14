use crate::exec::parser::ExecArgs;
use crate::interfaces::parser::Validation;
use crate::tscp::parser::TscpArgs;
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author = "@Leghart @WiktorNowak", version = "1.0.0", about)]
/// Main parser
pub struct AppArgs {
    #[clap(subcommand)]
    operation: Operation,

    /// Global flags
    #[clap(flatten)]
    global_opts: GlobalOpts,
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
    /// Copies data between two machines
    Tscp(TscpArgs),

    /// Execute command on machine.
    Exec(ExecArgs),
}

impl Validation for Operation {
    fn validate(&mut self) -> Result<(), crate::error::CrustError> {
        match self {
            Operation::Exec(exec_args) => {
                exec_args.validate()?;
            }
            Operation::Tscp(tscp_args) => {
                tscp_args.validate()?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Args)]
struct GlobalOpts {
    #[clap(long, short, global = true)]
    main_verbose: Option<usize>,
    //... other global options
}
