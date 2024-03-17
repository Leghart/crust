use std::fs::File;

use std::path::{Path, PathBuf};
use std::thread;

use ssh2::Session;

use crate::connection::SshConnection;
use crate::connection::SSH;
use crate::error::{CrustError, ExitCode};
use crate::interfaces::progress_bar::ProgressBar;
use crate::interfaces::response::CrustResult;
use crate::scp::{copy_data, TransferFile};

// TODO!: fix progress bar
pub fn upload(
    mut ssh: SshConnection,
    from: &Path,
    to: &Path,
    progress: bool,
    threads: Option<u8>,
) -> Result<CrustResult, CrustError> {
    let meta = std::fs::metadata(from)?;

    if !ssh.is_connected() {
        ssh.connect()?;
    }
    let session = ssh.session();

    if meta.is_file() {
        return _upload_file(session, from, to, progress);
    } else if meta.is_dir() {
        let sftp = session.sftp()?;

        match sftp.stat(to) {
            Ok(_) => {
                return Err(CrustError {
                    code: ExitCode::Remote,
                    message: format!("Directory '{to:?}' already exists"),
                })
            }
            Err(_) => sftp.mkdir(to, 0o755)?,
        };

        match threads {
            None => {
                for path in std::fs::read_dir(from)? {
                    let new_path_from = path?;
                    let new_path_to = Path::new(to).join(new_path_from.path().file_name().unwrap());
                    upload(
                        ssh.clone(),
                        &new_path_from.path(),
                        &new_path_to,
                        progress,
                        threads,
                    )?;
                }
            }
            Some(_t) => {
                // TODO!: add semaphore for max threads numer
                let handles: Vec<_> = std::fs::read_dir(from)?
                    .map(|path| {
                        let ssh = ssh.clone();
                        let to = PathBuf::from(&to);
                        thread::spawn(move || {
                            let new_path_from = path.unwrap();
                            let new_path_to =
                                Path::new(&to).join(new_path_from.path().file_name().unwrap());
                            upload(
                                ssh.clone(),
                                &new_path_from.path(),
                                &new_path_to,
                                progress,
                                threads,
                            )
                        })
                    })
                    .collect();

                for thread in handles {
                    if thread.join().is_err() {
                        return Err(CrustError {
                            code: ExitCode::Internal,
                            message: "Thread error".to_string(),
                        });
                    }
                }
            }
        };
    } else {
        return Err(CrustError {
            code: ExitCode::Local,
            message: format!("'{from:?}' source is not file or directory"),
        });
    }
    Ok(CrustResult::default())
}

fn _upload_file(
    sess: Session,
    from: &Path,
    to: &Path,
    progress: bool,
) -> Result<CrustResult, CrustError> {
    let size: u64 = match std::fs::metadata(from) {
        Ok(metadata) => metadata.len(),
        Err(_) => {
            return Err(CrustError {
                code: ExitCode::Local,
                message: "Can not get file size".to_string(),
            });
        }
    };

    let file_to_write = TransferFile::Remote(sess.scp_send(to, 0o644, size, None).unwrap());

    let file_to_read = TransferFile::Local(File::open(from).expect("Can not open file on local"));

    let progress_bar: Option<ProgressBar> = match progress {
        true => Some(ProgressBar::new(size)),
        false => None,
    };

    copy_data(file_to_read, file_to_write, progress_bar);

    Ok(CrustResult::default())
}
