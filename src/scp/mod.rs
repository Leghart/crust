pub mod parser;
use ssh2::Channel;

use crate::error::{CrustError, ExitCode};
use crate::interfaces::progress_bar::ProgressBar;
use crate::machine::base::{Machine, MachineType};

use std::io::Read;
use std::io::Write;

use std::fs::File;
use std::path::PathBuf;
pub const BUF_SIZE: usize = 8192;

/// Function enabling automatic selection of machines for
/// perform the requested operation.
pub fn scp(
    machine_from: &mut Box<dyn Machine>,
    machine_to: &mut Box<dyn Machine>,
    path_from: PathBuf,
    path_to: PathBuf,
    progress: bool,
) -> Result<(), CrustError> {
    match (machine_from.get_machine(), machine_to.get_machine()) {
        (MachineType::LocalMachine, MachineType::RemoteMachine) => {
            machine_from.copy(machine_to, path_from, path_to, progress)
        }
        (MachineType::RemoteMachine, MachineType::LocalMachine) => {
            machine_to.copy(machine_from, path_from, path_to, progress)
        }
        (MachineType::LocalMachine, MachineType::LocalMachine) => Err(CrustError {
            code: ExitCode::Local,
            message: "You want to copy files between local machines. Use 'exec' instead."
                .to_string(),
        }),
        (MachineType::RemoteMachine, MachineType::RemoteMachine) => todo!("will be done"),
        (_, _) => panic!("unsupported yet"),
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
    /// Universal method for copying data between two machines.
    /// Supports [Box<dyn Machine>] objects and results from MachinesManager as well.
    /// TODO!: add copy directories
    fn copy(
        &self,
        machine: &mut Box<dyn Machine>,
        from: PathBuf,
        to: PathBuf,
        progress: bool,
    ) -> Result<(), CrustError> {
        if let MachineType::RemoteMachine = machine.mtype() {
            machine.connect()?;
        }

        let size: u64;
        let mut file_to_read: TransferFile;
        let mut file_to_write: TransferFile;

        match machine.mtype() {
            MachineType::LocalMachine => {
                let (channel, stat) = machine.get_session().unwrap().scp_recv(from.as_path())?;
                size = stat.size();
                file_to_read = TransferFile::Remote(channel);
                file_to_write =
                    TransferFile::Local(File::create(to).expect("Failed to create file"));
            } //download
            MachineType::RemoteMachine => {
                size = match std::fs::metadata(&from) {
                    Ok(metadata) => metadata.len(),
                    Err(_) => {
                        return Err(CrustError {
                            code: ExitCode::Local,
                            message: "Can not get file size".to_string(),
                        });
                    }
                };
                file_to_read =
                    TransferFile::Local(File::open(&from).expect("Can not open file on local"));
                file_to_write = TransferFile::Remote(
                    machine
                        .get_session()
                        .unwrap()
                        .scp_send(to.as_path(), 0o644, size, None)
                        .unwrap(),
                );
            } //upload
            MachineType::AbstractMachine => unimplemented!(),
        };

        let progress_bar: Option<ProgressBar> = match progress {
            true => Some(ProgressBar::new(size)),
            false => None,
        };

        let mut buffer = [0; BUF_SIZE];
        loop {
            let len = file_to_read
                .read(&mut buffer)
                .expect("Failed to read from local file");

            if len == 0 {
                break;
            }

            file_to_write
                .write_all(&buffer[..len])
                .expect("Failed to write to file");

            if let Some(ref pb) = progress_bar {
                pb.inc(len);
            }
        }

        if let Some(pb) = progress_bar {
            pb.finish();
        }

        match (file_to_read, file_to_write) {
            (TransferFile::Remote(mut remote), _) | (_, TransferFile::Remote(mut remote)) => {
                remote.send_eof().unwrap();
                remote.wait_eof().unwrap();
                remote.close().unwrap();
                remote.wait_close().unwrap();
            }
            _ => {}
        }

        Ok(())
    }

    /// Getter for machine (common interface provided by Machine trait).
    fn get_machine(&self) -> MachineType;

    /// Getter for string preoresentation of machine. Used in
    /// connection in ssh2 crate.
    fn get_address(&self) -> String;
}
