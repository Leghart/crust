use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ssh2::Channel;

use crate::error::{CrustError, ExitCode};
use crate::interfaces::progress_bar::ProgressBar;
use crate::interfaces::response::CrustResult;
use crate::machine::local::LocalMachine;
use crate::machine::{Machine, MachineType};

pub mod parser;

pub const BUF_SIZE: usize = 1024 * 10;

/// Function enabling automatic selection of machines to
/// perform the requested operation.
/// TODO?: copying between two machines is done temporarily with the proxy machine
pub fn scp(
    _machine_from: &Rc<RefCell<Box<dyn Machine>>>,
    _machine_to: &Rc<RefCell<Box<dyn Machine>>>,
    path_from: PathBuf,
    path_to: PathBuf,
    progress: bool,
) -> Result<CrustResult, CrustError> {
    let mut machine_from = _machine_from.borrow_mut();
    let mut machine_to = _machine_to.borrow_mut();
    match (machine_from.get_machine(), machine_to.get_machine()) {
        (MachineType::LocalMachine, MachineType::RemoteMachine) => {
            log::trace!("Run `upload` from {} to {}", machine_from, machine_to);
            machine_from.upload(&mut machine_to, &path_from, &path_to, progress)
        }
        (MachineType::RemoteMachine, MachineType::LocalMachine) => {
            log::trace!("Run `download` from {} to {}", machine_to, machine_from);
            machine_to.download(&mut machine_from, &path_from, &path_to, progress)
        }
        (MachineType::RemoteMachine, MachineType::RemoteMachine) => {
            let mut local: Box<dyn Machine> = Box::<LocalMachine>::default();
            local.create_tmpdir()?;
            let file_path = local.create_tmpdir_content("tmp_scp")?;
            log::trace!("Run `download` from {} to {}", machine_from, local);
            local.download(&mut machine_from, &path_from, &file_path, progress)?;
            log::trace!("Run `upload` from {} to {}", local, machine_to);
            local.upload(&mut machine_to, &file_path, &path_to, progress)?;

            Ok(CrustResult::default())
        }
        (MachineType::LocalMachine, MachineType::LocalMachine) => Err(CrustError {
            code: ExitCode::Local,
            message: "You want to copy files between local machines. Use 'exec' instead."
                .to_string(),
        }),

        (_, _) => panic!("unsupported yet"),
    }
}

/// Private function for copying single-file data by bytes. Used by `upload`
/// and `download` methods.
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
    /// TODO: add copy directories
    /// Allows to upload resource from local to remote.
    /// Supports [Box<dyn Machine>] objects and results from MachinesManager as well.
    fn upload(
        &self,
        machine: &mut Box<dyn Machine>,
        from: &Path,
        to: &Path,
        progress: bool,
    ) -> Result<CrustResult, CrustError> {
        // Pseudo-code
        // if file_from.isfile(){
        //     copy_data(from,to);
        // } else{
        //     let dirname = from.name;
        //     if remote.has_dir(dirname){
        //         return Err?;
        //     }
        //     remote.mkdir(dirname);
        //     for content in local.from.listdir(){
        //         copy(from/dirname/content, to/dirname/content)
        //     }
        // }

        machine.connect()?;

        let size: u64 = match std::fs::metadata(from) {
            Ok(metadata) => metadata.len(),
            Err(_) => {
                return Err(CrustError {
                    code: ExitCode::Local,
                    message: "Can not get file size".to_string(),
                });
            }
        };

        let file_to_write = TransferFile::Remote(
            machine
                .get_session()
                .unwrap()
                .scp_send(to, 0o644, size, None)
                .unwrap(),
        );

        let file_to_read =
            TransferFile::Local(File::open(from).expect("Can not open file on local"));

        let progress_bar: Option<ProgressBar> = match progress {
            true => Some(ProgressBar::new(size)),
            false => None,
        };

        copy_data(file_to_read, file_to_write, progress_bar);

        Ok(CrustResult::default())
    }

    /// TODO: add copy directories
    /// Allows to download resource from remote to local.
    /// Supports [Box<dyn Machine>] objects and results from MachinesManager as well.
    fn download(
        &self,
        machine: &mut Box<dyn Machine>,
        from: &Path,
        to: &Path,
        progress: bool,
    ) -> Result<CrustResult, CrustError> {
        machine.connect()?;

        let (channel, stat) = machine.get_session().unwrap().scp_recv(from)?;
        let file_to_read = TransferFile::Remote(channel);
        let size = stat.size();

        let file_to_write =
            TransferFile::Local(std::fs::File::create(to).expect("Failed to create file"));

        let progress_bar: Option<ProgressBar> = match progress {
            true => Some(ProgressBar::new(size)),
            false => None,
        };

        copy_data(file_to_read, file_to_write, progress_bar);
        Ok(CrustResult::default())
    }

    /// Getter for machine (common interface provided by Machine trait).
    fn get_machine(&self) -> MachineType;
}
