use std::path::PathBuf;

use crate::error::{CrustError, ExitCode};
use crate::interfaces::parser::Validation;
use clap::Args;

/// Interface to sub struct with connection args.
pub trait BaseConnArgs {
    fn addr(&self) -> Option<&String>;
    fn port(&self) -> Option<u16>;
    fn password(&self) -> Option<&String>;
    fn pkey(&self) -> Option<&PathBuf>;
    fn alias(&self) -> Option<&String>;

    /// Split address to get user and host.
    /// Assumes that address was passed.
    fn split_addr(&self) -> (String, String) {
        let (u, h) = self.addr().as_ref().unwrap().split_once('@').unwrap();
        (u.to_string(), h.to_string())
    }
}

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

    #[clap(long)]
    /// Alias for remote machine to use instead of all passing all args
    pub alias_to: Option<String>,
}

impl BaseConnArgs for ConnectionArgsTo {
    fn addr(&self) -> Option<&String> {
        self.addr_to.as_ref()
    }
    fn alias(&self) -> Option<&String> {
        self.alias_to.as_ref()
    }
    fn password(&self) -> Option<&String> {
        self.password_to.as_ref()
    }
    fn pkey(&self) -> Option<&PathBuf> {
        self.pkey_to.as_ref()
    }
    fn port(&self) -> Option<u16> {
        self.port_to
    }
}

impl Validation for ConnectionArgsTo {
    fn validate(&mut self) -> Result<(), CrustError> {
        if self.alias_to.is_some() {
            return Ok(());
        }

        if self.password_to.is_none() && self.pkey_to.is_none() {
            return Err(CrustError {
                code: ExitCode::Parser,
                message: "Neither password nor pkey provided".to_string(),
            });
        }

        if let Some(addr) = &self.addr_to {
            let parts = addr.split('@').collect::<Vec<&str>>();
            if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
                return Err(CrustError {
                    code: ExitCode::Parser,
                    message: "Invalid address pattern. Use <user>@<host>".to_string(),
                });
            }
        }
        Ok(())
    }
}

/// Separated struct with connection data required by methods which
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

    #[clap(long)]
    /// Alias for remote machine to use instead of all passing all args
    pub alias_from: Option<String>,
}

impl BaseConnArgs for ConnectionArgsFrom {
    fn addr(&self) -> Option<&String> {
        self.addr_from.as_ref()
    }
    fn alias(&self) -> Option<&String> {
        self.alias_from.as_ref()
    }
    fn password(&self) -> Option<&String> {
        self.password_from.as_ref()
    }
    fn pkey(&self) -> Option<&PathBuf> {
        self.pkey_from.as_ref()
    }
    fn port(&self) -> Option<u16> {
        self.port_from
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
