use clap::Args;
use std::path::PathBuf;

use crate::error::{CrustError, ExitCode};
use crate::interfaces::parser::Validation;

// TODO: use a ConnectionArgs parser
#[derive(Args, Clone, Debug)]
#[clap()]
/// At least one of argument <password>|<pkey> must be provided to
/// connect to remote server.
pub struct ScpArgs {
    /// Source path (local or remote machine)
    pub src: String,

    /// Destination path (remote or local machine)
    pub dst: String,

    #[clap(short, long, default_value = "22")]
    /// Remote machine's port
    pub port: u16,

    #[clap(short, long, default_value = "false")]
    /// Disable logs for scp chunk results
    pub verbose: bool,

    #[clap(long)]
    /// Password to remote server. Shoudn't be used on production (only for tests)
    pub password: Option<String>,

    #[clap(long)]
    /// Path to private ssh-key to remote server.
    pub pkey: Option<PathBuf>,
}

impl Validation for ScpArgs {
    fn validate(&mut self) -> Result<(), CrustError> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct ValidatedArgs {
    pub src_path: String,
    pub dst_path: String,

    pub src_hostname: Option<String>,
    pub dst_hostname: Option<String>,

    pub src_username: Option<String>,
    pub dst_username: Option<String>,

    pub port: u16,
    pub password: Option<String>,
    pub pkey: Option<PathBuf>,

    pub verbose: bool,
}

/// Validates a passed arguments.
/// Methods in this case are not used as struct methods (they are more as
/// class methods), because I wanted to validate them before struct creation.
/// The problem is, that self reference does not exist yet.
impl ValidatedArgs {
    pub fn validate_and_create(raw_args: ScpArgs) -> Result<Self, CrustError> {
        ValidatedArgs::validate(&raw_args)?;

        let (src_username, src_hostname, src_path) = ValidatedArgs::unpack_address(&raw_args.src)?;
        let (dst_username, dst_hostname, dst_path) = ValidatedArgs::unpack_address(&raw_args.dst)?;

        Ok(Self {
            src_username,
            src_hostname,
            src_path,
            dst_username,
            dst_hostname,
            dst_path,

            password: raw_args.password,
            pkey: raw_args.pkey,
            port: raw_args.port,
            verbose: raw_args.verbose,
        })
    }

    /// Runs multiple single assertions.
    pub fn validate(data: &ScpArgs) -> Result<(), CrustError> {
        if !data.src.contains(':') && !data.dst.contains(':') {
            return Err(CrustError {
                code: ExitCode::Parser,
                message: "Passed two local servers".to_string(),
            });
        }

        if data.src.contains(':') && data.dst.contains(':') {
            return Err(CrustError {
                code: ExitCode::Parser,
                message: "Passed two remote servers".to_string(),
            });
        }

        if data.src.matches(':').count() > 1 || data.dst.matches(':').count() > 1 {
            return Err(CrustError {
                code: ExitCode::Parser,
                message: "Invalid colon amount in remote address".to_string(),
            });
        }

        if data.password.is_none() && data.pkey.is_none() {
            return Err(CrustError {
                code: ExitCode::Parser,
                message: "Neither password nor pkey provided".to_string(),
            });
        }

        Ok(())
    }

    fn get_user_host(address: &str) -> Result<(String, String), CrustError> {
        let incorrect_address_err = CrustError {
            code: ExitCode::Parser,
            message: "For remote server, provide address as <user>@<host>:<path>".to_string(),
        };

        let addr = match address.split(':').next() {
            Some(val) => val,
            None => return Err(incorrect_address_err),
        };

        if addr.matches('@').count() != 1 {
            return Err(incorrect_address_err);
        }

        let splitted_addr = addr.split('@').collect::<Vec<&str>>();
        let user = splitted_addr[0];
        let host = splitted_addr[1];

        if user.is_empty() || host.is_empty() {
            return Err(incorrect_address_err);
        }

        Ok((user.to_string(), host.to_string()))
    }

    /// Determine type of machine by passed destination.
    #[inline]
    fn is_local(server: &str) -> bool {
        !server.contains(':')
    }

    /// Gets a pure data which determines a machine type.
    /// Address argument must be already validated.
    /// In case of localmachine it is:
    ///  - None, None, path
    /// In case of remotemachine it is:
    ///  - Some(user), Some(host), path
    fn unpack_address(
        address: &str,
    ) -> Result<(Option<String>, Option<String>, String), CrustError> {
        let path = ValidatedArgs::get_path(address);
        let username: Option<String>;
        let hostname: Option<String>;

        if ValidatedArgs::is_local(address) {
            username = None;
            hostname = None;
        } else {
            let (_user, _host) = ValidatedArgs::get_user_host(address)?;
            username = Some(_user);
            hostname = Some(_host);
        }

        Ok((username, hostname, path))
    }

    /// Gets a pure path from destination argument.
    /// If it is a localmachine, then destination = path, otherwise
    /// gets a path from splitted dest.
    fn get_path(address: &str) -> String {
        if ValidatedArgs::is_local(address) {
            address.to_string()
        } else {
            let path = address.splitn(2, ':').collect::<Vec<&str>>().pop().unwrap();
            path.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ScpArgs, ValidatedArgs};

    #[test]
    fn test_is_local_machine() {
        assert_eq!(ValidatedArgs::is_local("user@host:path"), false);
        assert_eq!(ValidatedArgs::is_local(":path"), false);

        assert_eq!(ValidatedArgs::is_local("path"), true);
    }

    #[test]
    fn test_unpack_address_correct() {
        assert_eq!(
            ValidatedArgs::unpack_address("path").unwrap(),
            (None, None, String::from("path"))
        );
        assert_eq!(
            ValidatedArgs::unpack_address("user@host:path").unwrap(),
            (
                Some(String::from("user")),
                Some(String::from("host")),
                String::from("path")
            )
        );
    }

    #[test]
    fn test_validation_local_hosts() {
        let args = ScpArgs {
            src: String::from("local"),
            dst: String::from("local"),
            port: 22,
            verbose: false,
            password: None,
            pkey: None,
        };
        let result = ValidatedArgs::validate(&args).err().unwrap();

        assert_eq!(result.message, "Passed two local servers");
    }

    #[test]
    fn test_validation_remote_hosts() {
        let args = ScpArgs {
            src: String::from(":remote"),
            dst: String::from(":remote"),
            port: 22,
            verbose: false,
            password: None,
            pkey: None,
        };
        let result = ValidatedArgs::validate(&args).err().unwrap();

        assert_eq!(result.message, "Passed two remote servers");
    }

    #[test]
    fn test_validation_neither_pkey_password() {
        let args = ScpArgs {
            src: String::from("local"),
            dst: String::from(":remote"),
            port: 22,
            verbose: false,
            password: None,
            pkey: None,
        };
        let result = ValidatedArgs::validate(&args).err().unwrap();

        assert_eq!(result.message, "Neither password nor pkey provided");
    }
}
