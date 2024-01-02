use clap::Parser;
use regex::Regex;
use std::path::PathBuf;

use super::error::{CrustError, ExitCode};

#[derive(Parser, Debug)]
#[clap(author = "@Leghart @WiktorNowak", version = "1.0.0", about)]
///CLI for RSCP (multi-threaded scp in Rust).
/// At least one of argument <password>|<pkey> must be provided to
/// connect to remote server.
pub struct RawArgs {
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
    pub fn new(raw_args: RawArgs) -> Result<Self, CrustError> {
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
            threads: raw_args.threads,
            chunk_size: ValidatedArgs::str_to_usize(raw_args.chunk_size),
            verbose: raw_args.verbose,
        })
    }

    /// Runs multiple single assertions.
    pub fn validate(data: &RawArgs) -> Result<(), CrustError> {
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
    /// TODO: add assertion for overload usize
    fn str_to_usize(opt_str: Option<String>) -> Option<u64> {
        opt_str.as_ref()?;

        let size_str = opt_str.unwrap();
        let (numeric_part, unit) = size_str.split_at(size_str.len() - 1);
        let numeric_part = match numeric_part.parse::<u64>() {
            Ok(num) if num > 0 => num,
            Err(_) => return None,
            _ => return None,
        };

        match unit.to_ascii_uppercase().as_str() {
            "K" | "k" => Some(numeric_part * 1024),
            "M" => Some(numeric_part * 1024 * 1024),
            "G" => Some(numeric_part * 1024 * 1024 * 1024),
            _ => None,
        }
    }

    fn get_user_host(address: &str) -> Result<(String, String), CrustError> {
        let regex = Regex::new(r"^[^@]+@[^:]+:.+$").unwrap();
        if !regex.is_match(address) {
            return Err(CrustError {
                code: ExitCode::Parser,
                message: "Incorrect remote destination pattern".to_string(),
            });
        }

        let regex_user_host = Regex::new(r"([^@]+)@([^:]+):").unwrap();
        if let Some(captures) = regex_user_host.captures(address) {
            if let (Some(user), Some(host)) = (captures.get(1), captures.get(2)) {
                return Ok((user.as_str().to_string(), host.as_str().to_string()));
            } else {
                Err(CrustError {
                    code: ExitCode::Parser,
                    message: "Incorrect username and hostname patterns".to_string(),
                })
            }
        } else {
            Err(CrustError {
                code: ExitCode::Parser,
                message: "Incorrect remote address pattern".to_string(),
            })
        }
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
    use super::{RawArgs, ValidatedArgs};

    #[test]
    fn test_str_to_size() {
        assert_eq!(
            ValidatedArgs::str_to_usize(Some("5M".to_string())),
            Some(5242880)
        );
        assert_eq!(
            ValidatedArgs::str_to_usize(Some("100G".to_string())),
            Some(107374182400)
        );
        assert_eq!(
            ValidatedArgs::str_to_usize(Some("70k".to_string())),
            Some(71680)
        );
        assert_eq!(
            ValidatedArgs::str_to_usize(Some("1K".to_string())),
            Some(1024)
        );

        assert_eq!(ValidatedArgs::str_to_usize(Some("10T".to_string())), None);
        assert_eq!(ValidatedArgs::str_to_usize(Some("-2M".to_string())), None);
        assert_eq!(ValidatedArgs::str_to_usize(Some("test".to_string())), None);
        assert_eq!(ValidatedArgs::str_to_usize(Some("10B".to_string())), None);
        assert_eq!(ValidatedArgs::str_to_usize(Some("0k".to_string())), None);
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
        let args = RawArgs {
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
        let args = RawArgs {
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
        let args = RawArgs {
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
        let args = RawArgs {
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
        let args = RawArgs {
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
