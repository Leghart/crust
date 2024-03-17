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
use crate::scp::download::download;
use crate::scp::upload::upload;

pub mod download;
pub mod parser;
pub mod upload;

pub const BUF_SIZE: usize = 1024 * 10;

/// Function enabling automatic selection of machines to
/// perform the requested operation.
pub fn scp(
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

    if !machine_from.is_connected() {
        machine_from.connect()?;
    }

    match (machine_from.mtype(), machine_to.mtype()) {
        (MachineType::LocalMachine, MachineType::RemoteMachine) => {
            log::trace!("Run `upload` from {} to {}", machine_from, machine_to);
            let ssh = machine_to.get_ssh().unwrap();
            upload(ssh, &path_from, &path_to, progress)
        }
        (MachineType::RemoteMachine, MachineType::LocalMachine) => {
            log::trace!("Run `download` from {} to {}", machine_to, machine_from);
            let ssh = machine_from.get_ssh().unwrap();
            download(ssh, &path_from, &path_to, progress)
        }
        (MachineType::RemoteMachine, MachineType::RemoteMachine) => {
            let mut local: Box<dyn Machine> = Box::<LocalMachine>::default();
            local.create_tmpdir()?;
            let file_path = local.create_tmpdir_content("tmp_scp")?;

            log::trace!("Run `download` from {} to {}", machine_from, local);
            let ssh_from = machine_from.get_ssh().unwrap();
            download(ssh_from, &path_from, &file_path, progress)?;

            log::trace!("Run `upload` from {} to {}", local, machine_to);
            let ssh_to = machine_to.get_ssh().unwrap();
            upload(ssh_to, &file_path, &path_to, progress)?;

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
