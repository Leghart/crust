use std::path::PathBuf;

use crate::error::CrustError;
use crate::interfaces::parser::Validation;
use clap::Args;

#[derive(Debug, Args, Clone)]
pub struct ConnectionArgs {
    #[clap(long)]
    pub user: Option<String>,

    #[clap(long)]
    pub host: Option<String>,

    #[clap(long, default_value = "22")]
    pub port: u16,

    #[clap(long)]
    pub as_str: Option<String>,

    #[clap(long)]
    pub password: Option<String>,

    #[clap(long)]
    pub pkey: Option<PathBuf>,
}

impl Validation for ConnectionArgs {
    fn validate(&mut self) -> Result<(), CrustError> {
        if let Some(val) = self.as_str.as_ref() {
            match val.split_once('@') {
                Some((user, host)) => {
                    self.user = Some(user.to_string());
                    self.host = Some(host.to_string());
                }
                None => {
                    panic!("TODO")
                }
            }
        }
        Ok(())
    }
}
