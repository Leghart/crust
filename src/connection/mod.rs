pub mod manager;
pub mod parser;

use crate::exec::BUFF_SIZE;
use crate::interfaces::response::CrustResult;
use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::path::PathBuf;

use super::error::{CrustError, ExitCode};

/// Providing required methods for connecting to a remote server
pub trait SSH {
    fn new(
        username: &str,
        hostname: &str,
        private_key: Option<PathBuf>,
        password: Option<String>,
        port: u16,
    ) -> Self
    where
        Self: Sized;

    /// Remote version of `std::process::Command`.
    fn execute(&self, command: &str) -> Result<CrustResult, CrustError>;

    /// Remote version of execute (real-time).
    fn execute_rt(&self, command: &str, merge_pipes: bool) -> Result<CrustResult, CrustError>;

    /// Getter for current session
    fn session(&self) -> Session;

    /// Lazy method to connect to machine (creates a session)
    fn connect(&mut self) -> Result<(), CrustError>;

    /// Check if `connect()` was invoked and session was created.
    fn is_connected(&self) -> bool;
}

/// Represents arguments neccessary for connection.
#[derive(Clone)]
pub struct ConnectArgs {
    username: String,
    hostname: String,
    private_key: Option<PathBuf>,
    password: Option<String>,
    port: u16,
}

/// Main structure used in RemoteMachine
#[derive(Clone)]
pub struct SshConnection {
    session: Option<Session>,
    pub connect_args: Option<ConnectArgs>,
}

impl SSH for SshConnection {
    fn new(
        username: &str,
        hostname: &str,
        private_key: Option<PathBuf>,
        password: Option<String>,
        port: u16,
    ) -> Self {
        let connect_args = ConnectArgs {
            username: String::from(username),
            hostname: String::from(hostname),
            private_key,
            password,
            port,
        };
        Self {
            session: None,
            connect_args: Some(connect_args),
        }
    }

    fn session(&self) -> Session {
        self.session.clone().expect("Session was not created")
    }

    fn is_connected(&self) -> bool {
        match &self.session {
            None => false,
            Some(ses) => match ses.channel_session() {
                Ok(mut channel) => match channel.exec("") {
                    Ok(_) => {
                        let _ = channel.send_eof();
                        let _ = channel.wait_close();
                        true
                    }
                    Err(_) => false,
                },
                Err(_) => false,
            },
        }
    }

    fn connect(&mut self) -> Result<(), CrustError> {
        let conn_args = match &self.connect_args {
            Some(args) => args,
            None => {
                return Err(CrustError {
                    code: ExitCode::Ssh,
                    message: "Did not define connection arguments for session".to_string(),
                })
            }
        };

        let tcp = TcpStream::connect((conn_args.hostname.as_ref(), conn_args.port))?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;

        if let Some(pswd) = conn_args.password.as_ref() {
            log::debug!("Auth method - password");
            session.userauth_password(conn_args.username.as_str(), pswd.as_str())?;
        } else if let Some(pkey) = conn_args.private_key.as_ref() {
            log::debug!("Auth method - private key");
            session.userauth_pubkey_file(
                conn_args.username.as_str(),
                None,
                std::path::Path::new(&pkey),
                None,
            )?;
        } else {
            return Err(CrustError {
                code: ExitCode::Ssh,
                message: "Did not provide authorization. Neither password nor private key"
                    .to_string(),
            });
        }

        if !session.authenticated() {
            return Err(CrustError {
                code: ExitCode::Ssh,
                message: "Authentication failed".to_string(),
            });
        }
        log::debug!(
            "Session to '{}@{}' created",
            conn_args.username,
            conn_args.hostname
        );
        self.session = Some(session);
        Ok(())
    }

    fn execute(&self, command: &str) -> Result<CrustResult, CrustError> {
        let mut channel = self
            .session
            .as_ref()
            .expect("Call `.connect()` method first")
            .channel_session()?;

        channel.exec(command)?;

        let mut stdout = String::new();
        let mut stderr = String::new();
        let retcode = channel.exit_status()?;

        channel.read_to_string(&mut stdout)?;
        channel.stderr().read_to_string(&mut stderr)?;

        channel.wait_close()?;

        Ok(CrustResult::new(&stdout, &stderr, retcode))
    }

    fn execute_rt(&self, command: &str, merge_pipes: bool) -> Result<CrustResult, CrustError> {
        let mut channel = self
            .session
            .as_ref()
            .expect("Call `.connect()` method first")
            .channel_session()?;

        match merge_pipes {
            true => {
                channel.exec(&format!("{command} 2>&1"))?;

                let mut buffer = [0; BUFF_SIZE];

                loop {
                    let size = channel.read(&mut buffer)?;

                    if size == 0 {
                        break;
                    }

                    print!("{}", String::from_utf8(buffer[..size].to_vec())?);
                }
            }
            false => {
                channel.exec(command)?;

                let mut out_buffer = [0; BUFF_SIZE];
                let mut err_buffer = [0; BUFF_SIZE];

                loop {
                    let out_size = channel.read(&mut out_buffer)?;
                    let err_size = channel.stderr().read(&mut err_buffer)?;

                    if out_size == 0 && err_size == 0 {
                        break;
                    }

                    print!("{}", String::from_utf8(out_buffer[..out_size].to_vec())?);
                    log::error!("{}", String::from_utf8(err_buffer[..err_size].to_vec())?);
                }
            }
        };

        channel.wait_close()?;
        Ok(CrustResult::default())
    }
}

impl std::fmt::Display for SshConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let conn_args = self.connect_args.clone().unwrap();
        write!(f, "{}@{}", conn_args.username, conn_args.hostname)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connected_client() -> SshConnection {
        let mut ssh = SshConnection::new(
            "test_user",
            "10.10.10.10",
            None,
            Some(String::from("1234")),
            22,
        );
        let _ = ssh.connect();
        ssh
    }

    #[test]
    fn test_create_ssh_connection() {
        let ssh = SshConnection::new("username", "hostname", None, None, 22);

        assert!(ssh.session.is_none());
        assert!(ssh.connect_args.is_some());

        let args = ssh.connect_args.unwrap();
        assert_eq!(args.username, String::from("username"));
        assert_eq!(args.hostname, String::from("hostname"));
        assert_eq!(args.password, None);
        assert_eq!(args.private_key, None);
        assert_eq!(args.port, 22);
    }

    #[test]
    fn test_connect_no_args() {
        let mut ssh = SshConnection {
            connect_args: None,
            session: None,
        };
        let result = ssh.connect();

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.code, ExitCode::Ssh);
        assert_eq!(
            err.message,
            "Did not define connection arguments for session"
        );
    }

    #[test]
    fn test_connect_with_password() {
        let mut ssh = SshConnection::new(
            "test_user",
            "10.10.10.10",
            None,
            Some(String::from("1234")),
            22,
        );

        let result = ssh.connect();

        assert!(result.is_ok());
    }

    #[test]
    fn test_connect_with_pkey() {
        let mut ssh = SshConnection::new(
            "test_user",
            "10.10.10.10",
            Some(PathBuf::from("test_utils/rsa_keys/id_rsa")),
            None,
            22,
        );

        let result = ssh.connect();

        assert!(result.is_ok());
    }

    #[test]
    fn test_connect_with_no_auth() {
        let mut ssh = SshConnection::new("test_user", "10.10.10.10", None, None, 22);

        let result = ssh.connect();
        assert!(result.is_err());

        let err = result.err().unwrap();
        assert_eq!(err.code, ExitCode::Ssh);
        assert_eq!(
            err.message,
            "Did not provide authorization. Neither password nor private key"
        );
    }

    #[should_panic(expected = "Call `.connect()` method first")]
    #[test]
    fn test_execute_cmd_without_connection() {
        let ssh = SshConnection {
            session: None,
            connect_args: None,
        };

        let _ = ssh.execute("pwd");
    }

    #[test]
    fn test_execute_cmd_successful() {
        let ssh = connected_client();
        let result = ssh.execute("whoami");

        assert!(result.is_ok());

        let response = result.ok().unwrap();
        assert_eq!(response.stdout(), "test_user\n");
        assert_eq!(response.stderr(), "");
        assert_eq!(response.retcode(), 0);
    }

    #[should_panic(expected = "Call `.connect()` method first")]
    #[test]
    fn test_execute_rt_cmd_without_connection() {
        let ssh = SshConnection {
            session: None,
            connect_args: None,
        };

        let _ = ssh.execute_rt("pwd", false);
    }

    #[should_panic(expected = "Session was not created")]
    #[test]
    fn test_get_session_before_connect() {
        let ssh = SshConnection {
            session: None,
            connect_args: None,
        };
        let _ = ssh.session();
    }

    #[test]
    fn test_check_if_connected() {
        let mut ssh = SshConnection::new(
            "test_user",
            "10.10.10.10",
            None,
            Some(String::from("1234")),
            22,
        );

        assert_eq!(ssh.is_connected(), false);

        let result = ssh.connect();

        assert!(result.is_ok());
        assert_eq!(ssh.is_connected(), true);
    }

    #[test]
    fn test_display() {
        let ssh = connected_client();

        assert_eq!(format!("{ssh}"), "test_user@10.10.10.10");
    }
}
