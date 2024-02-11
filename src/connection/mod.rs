pub mod manager;
pub mod parser;

use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::path::PathBuf;

use super::error::{CrustError, ExitCode};

/// Providing required methods for connecting to a remote server
pub trait SSH {
    fn new(
        username: String,
        hostname: String,
        private_key: Option<PathBuf>,
        password: Option<String>,
        port: u16,
    ) -> Self
    where
        Self: Sized;

    /// Remote version of `std::process::Command`.
    fn execute(&self, command: &str) -> Result<String, CrustError>;

    /// Gets string representation of server address (username@hostname)
    fn ssh_address(&self) -> String;

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
        username: String,
        hostname: String,
        private_key: Option<PathBuf>,
        password: Option<String>,
        port: u16,
    ) -> Self {
        let connect_args = ConnectArgs {
            username,
            hostname,
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
        log::debug!("Session to '{:?}' created", self.ssh_address());
        self.session = Some(session);
        Ok(())
    }

    fn execute(&self, command: &str) -> Result<String, CrustError> {
        let mut channel = self
            .session
            .as_ref()
            .expect("Call `connect` method first")
            .channel_session()?;
        channel.exec(command)?;

        let mut stdout = String::new();
        let mut stderr = String::new();

        channel.read_to_string(&mut stdout)?;

        channel.stderr().read_to_string(&mut stderr)?;

        let status_code = channel.exit_status()?;

        let _ = channel.wait_close();

        if status_code != 0 {
            return Err(CrustError {
                code: ExitCode::Remote,
                message: stderr,
            });
        }

        Ok(stdout.trim().to_string())
    }

    fn ssh_address(&self) -> String {
        let conn_args = self
            .connect_args
            .clone()
            .expect("Did not provide connection arguments");
        format!("{}@{}", conn_args.username, conn_args.hostname)
    }
}
