use clap::Args;

use crate::connection::parser::ConnectionArgsTo;
use crate::error::CrustError;
use crate::interfaces::parser::Validation;

#[derive(Debug, Clone, Args)]
pub struct ExecArgs {
    /// Command to execute
    #[clap(value_delimiter = ' ', num_args = 1..)]
    pub cmd: Option<Vec<String>>,

    #[clap(flatten)]
    pub remote: Option<ConnectionArgsTo>,

    /// Collect output in real time mode
    #[clap(long, default_value = "false")]
    pub rt: bool,

    /// Merge streams (stderr into stdout)
    #[clap(short, long, default_value = "false")]
    pub merge: bool,
}

impl Validation for ExecArgs {
    fn validate(&mut self) -> Result<(), CrustError> {
        if self.remote.is_some() {
            self.remote.as_mut().unwrap().validate()?;
        }
        Ok(())
    }
}
