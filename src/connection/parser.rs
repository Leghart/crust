use std::path::PathBuf;

use crate::error::{CrustError, ExitCode};
use crate::interfaces::parser::Validation;
use clap::Args;

/// Struct with data required to connect to remote machine (default).
#[derive(Debug, Args, Clone)]
pub struct ConnectionArgsTo {
    #[clap(long)]
    /// Address to remote machine (<user>@<host>)
    pub addr_to: Option<String>,

    #[clap(long, default_value = "22")]
    /// Remote machine's port
    pub port_to: Option<u16>,

    #[clap(long)]
    /// Password to remote server
    pub password_to: Option<String>,

    #[clap(long)]
    /// Path to private ssh-key to remote server
    pub pkey_to: Option<PathBuf>,
}

impl ConnectionArgsTo {
    /// Gets 'username' and 'hostname' from `addr` field.
    pub fn split_addr(&self) -> (String, String) {
        let (u, h) = self.addr_to.as_ref().unwrap().split_once('@').unwrap();
        (u.to_string(), h.to_string())
    }
}

impl Validation for ConnectionArgsTo {
    fn validate(&mut self) -> Result<(), CrustError> {
        if let Some(addr) = &self.addr_to {
            let parts = addr.split('@').collect::<Vec<&str>>();
            if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
                return Err(CrustError {
                    code: ExitCode::Parser,
                    message: "Invalid address pattern. Use <user>@<host>".to_string(),
                });
            }

            if self.password_to.is_none() && self.pkey_to.is_none() {
                return Err(CrustError {
                    code: ExitCode::Parser,
                    message: "Neither password nor pkey provided".to_string(),
                });
            }
        }
        Ok(())
    }
}

/// Separeted struct with connection data required by methods which
/// use more than 1 remote machine.
/// As clap requires that every flag has a unique name, there is another
/// postfix `_from`.
#[derive(Debug, Args, Clone)]
pub struct ConnectionArgsFrom {
    #[clap(long)]
    /// Address to remote machine which is a source machine (<user>@<host>)
    pub addr_from: Option<String>,

    #[clap(long, default_value = "22")]
    /// Source remote machine's port
    pub port_from: Option<u16>,

    #[clap(long)]
    /// Password to source remote server
    pub password_from: Option<String>,

    #[clap(long)]
    /// Path to private ssh-key to source remote server
    pub pkey_from: Option<PathBuf>,
}

impl ConnectionArgsFrom {
    /// Gets 'username' and 'hostname' from `addr` field.
    pub fn split_addr(&self) -> (String, String) {
        let (u, h) = self.addr_from.as_ref().unwrap().split_once('@').unwrap();
        (u.to_string(), h.to_string())
    }
}

impl Validation for ConnectionArgsFrom {
    fn validate(&mut self) -> Result<(), CrustError> {
        if let Some(addr) = &self.addr_from {
            let parts = addr.split('@').collect::<Vec<&str>>();
            if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
                return Err(CrustError {
                    code: ExitCode::Parser,
                    message: "Invalid address pattern. Use <user>@<host>".to_string(),
                });
            }

            if self.password_from.is_none() && self.pkey_from.is_none() {
                return Err(CrustError {
                    code: ExitCode::Parser,
                    message: "Neither password nor pkey provided".to_string(),
                });
            }
        }
        Ok(())
    }
}
