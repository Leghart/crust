use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::path::PathBuf;

use super::error::{CrustError, ExitCode};

pub struct SshConnection {
    session: Option<Session>,
    username: String,
    hostname: String,
    private_key: Option<PathBuf>,
    password: Option<String>,
    port: u16,
}

impl SshConnection {
    pub fn new(
        username: String,
        hostname: String,
        private_key: Option<PathBuf>,
        password: Option<String>,
        port: u16,
    ) -> Self {
        Self {
            session: None,
            username,
            hostname,
            private_key,
            password,
            port,
        }
    }

    pub fn connect(&mut self) -> Result<(), CrustError> {
        if self.session.is_some() {
            return Ok(());
        }

        let tcp = TcpStream::connect((self.hostname.as_ref(), self.port))?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;

        if let Some(pswd) = self.password.as_ref() {
            session.userauth_password(self.username.as_str(), pswd.as_str())?;
        } else if let Some(pkey) = self.private_key.as_ref() {
            session.userauth_pubkey_file(
                self.username.as_str(),
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

        self.session = Some(session);
        Ok(())
    }

    pub fn exec(&self, command: &str) -> Result<String, CrustError> {
        let mut channel = self
            .session
            .as_ref()
            .expect("Call `connect` first")
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

    pub fn ssh_address(&self) -> String {
        format!("{}@{}", self.username, self.hostname)
    }
}
