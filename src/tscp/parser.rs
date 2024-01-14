use clap::Args;
use std::path::PathBuf;

use crate::error::{CrustError, ExitCode};
use crate::interfaces::parser::Validation;
use crate::machine::base::Machine;

// TODO: use a ConnectionArgs parser
#[derive(Args, Clone, Debug)]
#[clap()]
/// At least one of argument <password>|<pkey> must be provided to
/// connect to remote server.
pub struct TscpArgs {
    /// Source path (local or remote machine)
    pub src: String,

    /// Destination path (remote or local machine)
    pub dst: String,

    #[clap(short, long, default_value = "22")]
    /// Remote machine's port
    pub port: u16,

    #[clap(short, long)]
    /// Max chunk size
    pub chunk_size: Option<String>,

    #[clap(short, long)]
    /// Threads number
    pub threads: Option<u16>,

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

impl Validation for TscpArgs {
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

    pub threads: Option<u16>,
    pub chunk_size: Option<u64>,

    pub verbose: bool,
}

/// Validates a passed arguments.
/// Methods in this case are not used as struct methods (they are more as
/// class methods), because I wanted to validate them before struct creation.
/// The problem is, that self reference does not exist yet.
impl ValidatedArgs {
    pub fn validate_and_create(raw_args: TscpArgs) -> Result<Self, CrustError> {
        ValidatedArgs::validate(&raw_args)?;

        let (src_username, src_hostname, src_path) = ValidatedArgs::unpack_address(&raw_args.src)?;
        let (dst_username, dst_hostname, dst_path) = ValidatedArgs::unpack_address(&raw_args.dst)?;

        let parsed_chunks_size = match raw_args.chunk_size {
            Some(val) => Some(ValidatedArgs::str_to_usize(val)?),
            None => None,
        };

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
            threads: raw_args.threads,
            chunk_size: parsed_chunks_size,
            verbose: raw_args.verbose,
        })
    }

    pub fn get_split_size(&self, machine: &impl Machine) -> u64 {
        match self.chunk_size {
            Some(v) => v,
            None => {
                let threads = self.threads.unwrap() as u64;
                let cmd = format!("du -b {}", self.src_path);

                let total_size: u64 = machine
                    .exec(cmd.as_str())
                    .expect("Can not get size of original file")
                    .split_whitespace()
                    .next()
                    .expect("File size can not be determined")
                    .parse()
                    .expect("File size can not be converted to an integer");

                let chunk_size = total_size / threads;
                if total_size % threads == 0 {
                    chunk_size
                } else {
                    chunk_size + 1
                }
            }
        }
    }

    /// Runs multiple single assertions.
    pub fn validate(data: &TscpArgs) -> Result<(), CrustError> {
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

        if data.chunk_size.is_none() && data.threads.is_none() {
            return Err(CrustError {
                code: ExitCode::Parser,
                message: "Neither threads nor chunks provided".to_string(),
            });
        }

        if data.chunk_size.is_some() && data.threads.is_some() {
            return Err(CrustError {
                code: ExitCode::Parser,
                message: "Both threads and chunks entered".to_string(),
            });
        }

        if data.threads.is_some_and(|x| x == 0) {
            return Err(CrustError {
                code: ExitCode::Parser,
                message: "Thread number can not be equal to 0".to_string(),
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

    /// Changes passed string from human readable notation to usize.
    /// Possible formats:
    ///  - k | K -kilobytes
    ///  - M - megabytes
    ///  - G - gigabytes
    fn str_to_usize(size_str: String) -> Result<u64, CrustError> {
        let (numeric_part, unit) = size_str.split_at(size_str.len() - 1);
        let numeric_part = match numeric_part.parse::<u64>() {
            Ok(num) if num > 0 => num,
            Err(_) => {
                return Err(CrustError {
                    code: ExitCode::Parser,
                    message: "Could not parse chunk size number part to u64".to_string(),
                })
            }
            _ => {
                return Err(CrustError {
                    code: ExitCode::Parser,
                    message: "Int part of chunk size is <= 0".to_string(),
                })
            }
        };

        if numeric_part.checked_mul(1024u64.pow(3)).is_none() {
            return Err(CrustError {
                code: ExitCode::Parser,
                message: "Overflowed integer part of chunk size".to_string(),
            });
        }

        match unit.to_ascii_uppercase().as_str() {
            "K" | "k" => Ok(numeric_part * 1024),
            "M" => Ok(numeric_part * 1024 * 1024),
            "G" => Ok(numeric_part * 1024 * 1024 * 1024),
            _ => Err(CrustError {
                code: ExitCode::Parser,
                message: "Unknown numeric sign".to_string(),
            }),
        }
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
    use super::{TscpArgs, ValidatedArgs};

    #[test]
    fn test_str_to_size() {
        assert_eq!(
            ValidatedArgs::str_to_usize("5M".to_string()).ok().unwrap(),
            5242880
        );
        assert_eq!(
            ValidatedArgs::str_to_usize("100G".to_string())
                .ok()
                .unwrap(),
            107374182400
        );
        assert_eq!(
            ValidatedArgs::str_to_usize("70k".to_string()).ok().unwrap(),
            71680
        );
        assert_eq!(
            ValidatedArgs::str_to_usize("1K".to_string()).ok().unwrap(),
            1024
        );
        assert_eq!(
            ValidatedArgs::str_to_usize("10T".to_string())
                .err()
                .unwrap()
                .message,
            "Unknown numeric sign"
        );
        assert_eq!(
            ValidatedArgs::str_to_usize("-2M".to_string())
                .err()
                .unwrap()
                .message,
            "Could not parse chunk size number part to u64"
        );
        assert_eq!(
            ValidatedArgs::str_to_usize("test".to_string())
                .err()
                .unwrap()
                .message,
            "Could not parse chunk size number part to u64"
        );
        assert_eq!(
            ValidatedArgs::str_to_usize("0k".to_string())
                .err()
                .unwrap()
                .message,
            "Int part of chunk size is <= 0"
        );
    }

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
        let args = TscpArgs {
            src: String::from("local"),
            dst: String::from("local"),
            port: 22,
            chunk_size: None,
            threads: None,
            verbose: false,
            password: None,
            pkey: None,
        };
        let result = ValidatedArgs::validate(&args).err().unwrap();

        assert_eq!(result.message, "Passed two local servers");
    }

    #[test]
    fn test_validation_remote_hosts() {
        let args = TscpArgs {
            src: String::from(":remote"),
            dst: String::from(":remote"),
            port: 22,
            chunk_size: None,
            threads: None,
            verbose: false,
            password: None,
            pkey: None,
        };
        let result = ValidatedArgs::validate(&args).err().unwrap();

        assert_eq!(result.message, "Passed two remote servers");
    }

    #[test]
    fn test_validation_neither_size_threads() {
        let args = TscpArgs {
            src: String::from("local"),
            dst: String::from(":remote"),
            port: 22,
            chunk_size: None,
            threads: None,
            verbose: false,
            password: None,
            pkey: None,
        };
        let result = ValidatedArgs::validate(&args).err().unwrap();

        assert_eq!(result.message, "Neither threads nor chunks provided");
    }

    #[test]
    fn test_validation_both_size_threads() {
        let args = TscpArgs {
            src: String::from("local"),
            dst: String::from(":remote"),
            port: 22,
            chunk_size: Some(String::from("5M")),
            threads: Some(5),
            verbose: false,
            password: None,
            pkey: None,
        };
        let result = ValidatedArgs::validate(&args).err().unwrap();

        assert_eq!(result.message, "Both threads and chunks entered");
    }
    #[test]
    fn test_validation_neither_pkey_password() {
        let args = TscpArgs {
            src: String::from("local"),
            dst: String::from(":remote"),
            port: 22,
            chunk_size: Some(String::from("5M")),
            threads: None,
            verbose: false,
            password: None,
            pkey: None,
        };
        let result = ValidatedArgs::validate(&args).err().unwrap();

        assert_eq!(result.message, "Neither password nor pkey provided");
    }
}
