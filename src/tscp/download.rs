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

pub fn download(
    chunks: Vec<PathBuf>,
    src_machine: &RemoteMachine,
    dst_machine: &LocalMachine,
) -> Result<(), CrustError> {
    let handles: Vec<_> = chunks
        .into_iter()
        .map(|file_path| {
            let src_machine = src_machine.clone();
            let dst_machine = dst_machine.clone();

            thread::spawn(move || {
                let path_to = path_from_chunk(&file_path, dst_machine.get_tmpdir());
                transfer_download(src_machine, file_path.clone(), path_to)
            })
        })
        .collect();

    for handle in handles {
        if handle.join().is_err() {
            return Err(CrustError {
                code: ExitCode::Internal,
                message: "Thread [download] error occured".to_string(),
            });
        }
    }

    Ok(())
}

fn transfer_download(
    machine: RemoteMachine,
    from_path: PathBuf,
    to_path: PathBuf,
) -> Result<(), CrustError> {
    let mut machine = machine.clone();
    machine.connect()?;

    let (mut remote_file, _) = machine
        .get_session()
        .unwrap()
        .scp_recv(from_path.as_path())?;

    let mut file = std::fs::File::create(to_path).expect("Failed to create file");
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
