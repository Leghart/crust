use clap::Args;

use crate::connection::parser::{ConnectionArgsFrom, ConnectionArgsTo};
use crate::error::CrustError;
use crate::interfaces::parser::Validation;

/// Proxy struct to represent a source machine.
#[derive(Debug, Args, Clone)]
pub struct ScpConnectionArgsFrom {
    pub path_from: String,

    #[clap(flatten)]
    pub remote_params: Option<ConnectionArgsFrom>,
}

impl Validation for ScpConnectionArgsFrom {
    fn validate(&mut self) -> Result<(), CrustError> {
        if self.remote_params.is_some() {
            self.remote_params.as_mut().unwrap().validate()?;
        }
        Ok(())
    }
}

/// Proxy struct to represent a target machine.
#[derive(Debug, Args, Clone)]
pub struct ScpConnectionArgsTo {
    pub path_to: String,

    #[clap(flatten)]
    pub remote_params: Option<ConnectionArgsTo>,
}

impl Validation for ScpConnectionArgsTo {
    fn validate(&mut self) -> Result<(), CrustError> {
        if self.remote_params.is_some() {
            self.remote_params.as_mut().unwrap().validate()?;
        }
        Ok(())
    }
}

#[derive(Args, Clone, Debug)]
/// At least one of argument <password>|<pkey> must be provided to
/// connect to remote server.
pub struct ScpArgs {
    #[clap(flatten)]
    /// Source path (local or remote machine)
    pub src: ScpConnectionArgsFrom,

    #[clap(flatten)]
    /// Destination path (remote or local machine)
    pub dst: ScpConnectionArgsTo,

    #[clap(long, default_value = "false")]
    /// Show progress bar
    pub progress: bool,
}

impl Validation for ScpArgs {
    fn validate(&mut self) -> Result<(), CrustError> {
        self.src.validate()?;
        self.dst.validate()?;
        Ok(())
    }
}
