pub mod parser;
use crate::error::{CrustError, ExitCode};
use crate::machine::base::{Machine, MachineType};

use std::io::Read;
use std::io::Write;

use std::path::PathBuf;
pub const BUF_SIZE: usize = 4096;

/// Function enabling automatic selection of machines for
/// perform the requested operation.
pub fn scp(
    machine_from: &mut Box<dyn Machine>,
    machine_to: &mut Box<dyn Machine>,
    path_from: PathBuf,
    path_to: PathBuf,
) -> Result<(), CrustError> {
    match (machine_from.get_machine(), machine_to.get_machine()) {
        (MachineType::LocalMachine, MachineType::RemoteMachine) => {
            machine_from.upload(machine_to, path_from, path_to)
        }
        (MachineType::RemoteMachine, MachineType::LocalMachine) => {
            machine_to.download(machine_from, path_from, path_to)
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
pub trait Scp {
    /// TODO!: add copy directories
    /// Allows to upload resource from local to remote.
    /// Supports [Box<dyn Machine>] objects and results from MachinesManager as well.
    fn upload(
        &self,
        machine: &mut Box<dyn Machine>,
        from: PathBuf,
        to: PathBuf,
    ) -> Result<(), CrustError> {
        if let MachineType::RemoteMachine = machine.mtype() {
            machine.connect()?;
        }

        let size: u64 = match std::fs::metadata(&from) {
            Ok(metadata) => metadata.len(),
            Err(_) => {
                return Err(CrustError {
                    code: ExitCode::Local,
                    message: "Can not get file size".to_string(),
                });
            }
        };

        let mut remote_file = machine
            .get_session()
            .unwrap()
            .scp_send(to.as_path(), 0o644, size, None)
            .unwrap();

        let mut local_file = std::fs::File::open(&from).unwrap();

        let mut buffer = [0; BUF_SIZE];
        loop {
            let len = local_file
                .read(&mut buffer)
                .expect("Failed to read from channel");
            if len == 0 {
                break;
            }
            remote_file
                .write_all(&buffer[..len])
                .expect("Failed to write to file");
        }

        remote_file.send_eof().unwrap();
        remote_file.wait_eof().unwrap();
        remote_file.close().unwrap();
        remote_file.wait_close().unwrap();

        Ok(())
    }

    /// TODO!: add copy directories
    /// Allows to download resource from remote to local.
    /// Supports [Box<dyn Machine>] objects and results from MachinesManager as well.
    fn download(
        &self,
        machine: &mut Box<dyn Machine>,
        from: PathBuf,
        to: PathBuf,
    ) -> Result<(), CrustError> {
        machine.connect()?;

        let (mut remote_file, _) = machine.get_session().unwrap().scp_recv(from.as_path())?;

        let mut file = std::fs::File::create(to).expect("Failed to create file");
        let mut buffer = [0; BUF_SIZE];

        loop {
            let len = remote_file
                .read(&mut buffer)
                .expect("Failed to read from channel");
            if len == 0 {
                break;
            }
            file.write_all(&buffer[..len])
                .expect("Failed to write to file");
        }

        remote_file.send_eof().unwrap();
        remote_file.wait_eof().unwrap();
        remote_file.close().unwrap();
        remote_file.wait_close().unwrap();

        Ok(())
    }

    /// Getter for machine (common interface provided by Machine trait).
    fn get_machine(&self) -> MachineType;

    /// Getter for string preoresentation of machine. Used in
    /// connection in ssh2 crate.
    fn get_address(&self) -> String;
}
