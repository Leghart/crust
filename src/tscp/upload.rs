use std::io::Read;
use std::io::Write;

use std::path::PathBuf;
use std::thread;

use crate::error::{CrustError, ExitCode};

use super::BUF_SIZE;
use crate::interfaces::tmpdir::TemporaryDirectory;
use crate::machine::base::Machine;
use crate::machine::local::LocalMachine;
use crate::machine::remote::RemoteMachine;

use super::utils::path_from_chunk;

pub fn upload(
    chunks: Vec<PathBuf>,
    _: &LocalMachine,
    dst_machine: &RemoteMachine,
) -> Result<(), CrustError> {
    let handles: Vec<_> = chunks
        .into_iter()
        .map(|file_path| {
            let dst_machine = dst_machine.clone();

            thread::spawn(move || {
                let path_to = path_from_chunk(&file_path, dst_machine.get_tmpdir());
                transfer_upload(dst_machine, file_path.clone(), path_to)
            })
        })
        .collect();

    for handle in handles {
        if handle.join().is_err() {
            return Err(CrustError {
                code: ExitCode::Internal,
                message: "Thread [upload] error occured".to_string(),
            });
        }
    }
    Ok(())
}

fn transfer_upload(
    machine: RemoteMachine,
    from_path: PathBuf,
    to_path: PathBuf,
) -> Result<(), CrustError> {
    let mut machine = machine.clone();
    machine.connect()?;

    let size: u64 = match std::fs::metadata(&from_path) {
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
        .scp_send(to_path.as_path(), 0o644, size, None)
        .unwrap();

    let mut local_file = std::fs::File::open(&from_path).unwrap();

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
