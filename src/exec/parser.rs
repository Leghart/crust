use crate::connection::parser::ConnectionArgs;
use crate::error::CrustError;
use crate::interfaces::parser::Validation;
use clap::Args;

#[derive(Debug, Clone, Args)]
pub struct ExecArgs {
    /// Command to execute
    pub cmd: String,

    #[clap(flatten)]
    pub remote: Option<ConnectionArgs>,

    /// Collect output in real time mode
    #[clap(long, default_value = "false")]
    pub rt: bool,
    // More options
}

impl Validation for ExecArgs {
    fn validate(&mut self) -> Result<(), CrustError> {
        if self.remote.is_some() {
            self.remote.as_mut().unwrap().validate()?;
        }
        Ok(())
    }
}
