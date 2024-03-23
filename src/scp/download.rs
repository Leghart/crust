use std::path::{Path, PathBuf};
use std::thread;

use indicatif::{MultiProgress, ProgressBar};
use ssh2::Session;

use crate::connection::SshConnection;
use crate::connection::SSH;
use crate::error::{CrustError, ExitCode};
use crate::interfaces::progress_bar::set_custom_style;
use crate::interfaces::response::CrustResult;
use crate::scp::{copy_data, TransferFile};

/// Copies data from remote source machine to local machine (download).
/// Allows to copy file and directories (including nested structures).
pub fn download(
    mut ssh: SshConnection,
    from: &Path,
    to: &Path,
    multibars: Option<MultiProgress>,
    threads: Option<u8>,
) -> Result<CrustResult, CrustError> {
    if !ssh.is_connected() {
        ssh.connect()?;
    }
    let session = ssh.session();

    let sftp = session.sftp()?;
    match sftp.stat(from) {
        Err(_) => {
            return Err(CrustError {
                code: ExitCode::Remote,
                message: format!("Requested source '{from:?}' does not exist"),
            })
        }
        Ok(metadata) => {
            if metadata.is_file() {
                return _download_file(session, from, to, multibars);
            } else if metadata.is_dir() {
                match to.exists() {
                    true => {
                        return Err(CrustError {
                            code: ExitCode::Local,
                            message: format!("Directory '{to:?}' already exists"),
                        })
                    }
                    false => std::fs::create_dir(to),
                }?;

                match threads {
                    None => {
                        for (path, _) in sftp.readdir(from)? {
                            download(
                                ssh.clone(),
                                &path,
                                &Path::new(to).join(path.file_name().unwrap()),
                                multibars.clone(),
                                threads,
                            )?;
                        }
                    }
                    Some(_t) => {
                        // TODO?: add semaphore for max threads
                        let handles: Vec<_> = sftp
                            .readdir(from)?
                            .into_iter()
                            .map(|(path, _)| {
                                let ssh = ssh.clone();
                                let to = PathBuf::from(&to);
                                let multi = multibars.clone();

                                thread::spawn(move || {
                                    let new_path_from = path;
                                    let new_path_to =
                                        Path::new(&to).join(new_path_from.file_name().unwrap());
                                    download(
                                        ssh.clone(),
                                        &new_path_from,
                                        &new_path_to,
                                        multi,
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
                    code: ExitCode::Remote,
                    message: format!("'{from:?}' source is not file or directory"),
                });
            }
        }
    }
    Ok(CrustResult::default())
}

/// Collect data about source file and prepare to download data.
/// Supports [Box<dyn Machine>] objects and results from MachinesManager as well.
fn _download_file(
    session: Session,
    from: &Path,
    to: &Path,
    multibars: Option<MultiProgress>,
) -> Result<CrustResult, CrustError> {
    let (channel, stat) = session.scp_recv(from)?;
    let file_to_read = TransferFile::Remote(channel);
    let size = stat.size();

    let file_to_write =
        TransferFile::Local(std::fs::File::create(to).expect("Failed to create file"));

    let progress_bar = match multibars {
        Some(m) => {
            let pb = m.add(ProgressBar::new(size));
            set_custom_style(&pb);
            Some(pb)
        }
        None => None,
    };

    copy_data(file_to_read, file_to_write, progress_bar);
    Ok(CrustResult::default())
}
