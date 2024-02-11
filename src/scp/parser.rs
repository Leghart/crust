use clap::Args;
use std::path::PathBuf;

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
#[clap()]
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

#[derive(Debug)]
pub struct ValidatedArgs {
    pub path_from: String,
    pub username_from: Option<String>,
    pub hostname_from: Option<String>,
    pub port_from: Option<u16>,
    pub password_from: Option<String>,
    pub pkey_from: Option<PathBuf>,

    pub path_to: String,
    pub username_to: Option<String>,
    pub hostname_to: Option<String>,
    pub port_to: Option<u16>,
    pub password_to: Option<String>,
    pub pkey_to: Option<PathBuf>,

    pub progress: bool,
}

/// Validates a passed arguments.
/// Methods in this case are not used as struct methods (they are more as
/// class methods), because I wanted to validate them before struct creation.
/// The problem is, that self reference does not exist yet.
impl ValidatedArgs {
    pub fn validate_and_create(raw_args: ScpArgs) -> Result<Self, CrustError> {
        let port_from: Option<u16>;
        let pkey_from: Option<PathBuf>;
        let password_from: Option<String>;
        let username_from: Option<String>;
        let hostname_from: Option<String>;
        match raw_args.src.remote_params {
            Some(args_from) => {
                let (_u, _h) = args_from.split_addr();
                username_from = Some(_u);
                hostname_from = Some(_h);
                port_from = args_from.port_from;
                pkey_from = args_from.pkey_from;
                password_from = args_from.password_from;
            }
            _ => {
                username_from = None;
                hostname_from = None;
                port_from = None;
                pkey_from = None;
                password_from = None;
            }
        }

        let port_to: Option<u16>;
        let pkey_to: Option<PathBuf>;
        let password_to: Option<String>;
        let username_to: Option<String>;
        let hostname_to: Option<String>;
        match raw_args.dst.remote_params {
            Some(args_to) => {
                let (_u, _h) = args_to.split_addr();
                username_to = Some(_u);
                hostname_to = Some(_h);
                port_to = args_to.port_to;
                pkey_to = args_to.pkey_to;
                password_to = args_to.password_to;
            }
            _ => {
                username_to = None;
                hostname_to = None;
                port_to = None;
                pkey_to = None;
                password_to = None;
            }
        }

        Ok(Self {
            path_from: raw_args.src.path_from,
            username_from,
            hostname_from,
            port_from,
            password_from,
            pkey_from,
            path_to: raw_args.dst.path_to,
            username_to,
            hostname_to,
            port_to,
            password_to,
            pkey_to,
            progress: raw_args.progress,
        })
    }
}
