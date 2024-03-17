use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::thread;

use ssh2::Channel;
use ssh2::Session;

use crate::connection::SshConnection;
use crate::connection::SSH;
use crate::error::{CrustError, ExitCode};
use crate::interfaces::progress_bar::ProgressBar;
use crate::interfaces::response::CrustResult;
use crate::machine::local::LocalMachine;
use crate::machine::{Machine, MachineType};

pub const BUF_SIZE: usize = 1024 * 10;

/// Function enabling automatic selection of machines to
/// perform the requested operation.
/// TODO?: copying between two machines is done temporarily with the proxy machine
pub fn tscp(
    _machine_from: &Rc<RefCell<Box<dyn Machine>>>,
    _machine_to: &Rc<RefCell<Box<dyn Machine>>>,
    path_from: PathBuf,
    path_to: PathBuf,
    progress: bool,
) -> Result<CrustResult, CrustError> {
    let mut machine_from = _machine_from.borrow_mut();
    let mut machine_to = _machine_to.borrow_mut();

    if !machine_to.is_connected() {
        machine_to.connect()?;
    }
    let ssh = machine_to.get_ssh().unwrap();
    // let remote_sess = machine_to.get_session().unwrap();

    match (machine_from.machine_type(), machine_to.machine_type()) {
        (MachineType::LocalMachine, MachineType::RemoteMachine) => {
            log::trace!("Run `upload` from {} to {}", machine_from, machine_to);
            upload(ssh, &path_from, &path_to, progress)
        }

        (_, _) => panic!("unsupported yet"),
    }
}

/// Private function for copying single-file data by bytes. Used by `_upload_file`
/// and `_download_file` trait methods.
fn copy_data(
    mut file_source: TransferFile,
    mut file_target: TransferFile,
    progress_bar: Option<ProgressBar>,
) {
    let mut buffer = [0; BUF_SIZE];
    loop {
        let len = file_source
            .read(&mut buffer)
            .expect("Failed to read from local file");

        if len == 0 {
            break;
        }

        file_target
            .write_all(&buffer[..len])
            .expect("Failed to write to file");

        if let Some(ref pb) = progress_bar {
            pb.inc(len);
        }
    }

    if let Some(pb) = progress_bar {
        pb.finish();
    }

    match (file_source, file_target) {
        (TransferFile::Remote(mut remote), _) | (_, TransferFile::Remote(mut remote)) => {
            remote.send_eof().unwrap();
            remote.wait_eof().unwrap();
            remote.close().unwrap();
            remote.wait_close().unwrap();
        }
        _ => {}
    }
}

fn upload(
    mut ssh: SshConnection,
    from: &Path,
    to: &Path,
    progress: bool,
) -> Result<CrustResult, CrustError> {
    let meta = std::fs::metadata(from)?;

    if !ssh.is_connected() {
        println!("connecting");
        ssh.connect()?;
    }
    let sess = ssh.session();

    if meta.is_file() {
        return _upload_file(sess, from, to, progress);
    } else if meta.is_dir() {
        let sftp = sess.sftp()?;

        match sftp.stat(to) {
            Ok(_) => {
                return Err(CrustError {
                    code: ExitCode::Remote,
                    message: format!("Directory '{to:?}' already exists"),
                })
            }
            Err(_) => sftp.mkdir(to, 0o755)?,
        };

        let nto = PathBuf::from(&to);
        let threads: Vec<_> = std::fs::read_dir(from)?
            .into_iter()
            .map(|path| {
                let ssh = ssh.clone();
                let to = nto.clone();
                thread::spawn(move || {
                    let new_path_from = path.unwrap();
                    let new_path_to =
                        Path::new(&to).join(new_path_from.path().file_name().unwrap());
                    upload(ssh.clone(), &new_path_from.path(), &new_path_to, progress)
                })
            })
            .collect();
        for t in threads {
            if t.join().is_err() {
                return Err(CrustError {
                    code: ExitCode::Internal,
                    message: "Thread error".to_string(),
                });
            }
        }
        // for path in std::fs::read_dir(from)? {
        //     let new_path_from = path?;
        //     let new_path_to = Path::new(to).join(new_path_from.path().file_name().unwrap());
        //     upload(sess.clone(), &new_path_from.path(), &new_path_to, progress)?;
        // }
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

/// Represents a file which is source to get data in copy method.
/// In 'download' case it relates to Channel from remote machine, in
/// 'upload' it is a file located on local machine.
enum TransferFile {
    Remote(Channel),
    Local(File),
}

/// Allows common interface in copy method.
impl TransferFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        match self {
            TransferFile::Remote(channel) => channel.read(buf),
            TransferFile::Local(file) => file.read(buf),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
        match self {
            TransferFile::Remote(channel) => channel.write_all(buf),
            TransferFile::Local(file) => file.write_all(buf),
        }
    }
}

pub trait Scp {
    /// Collect data about source file and prepare to upload data.
    /// Supports [Box<dyn Machine>] objects and results from MachinesManager as well.

    /// Getter for machine type (common interface provided by Machine trait).
    fn machine_type(&self) -> MachineType;
}
