use clap::Parser;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

pub mod connection;
pub mod error;
pub mod machine;
pub mod parser;

use machine::base::{Machine, MachineType};
use machine::local::LocalMachine;
use machine::remote::RemoteMachine;

use error::{handle_result, CrustError, DefaultExitHandler};

/// Entrypoint for crust.
/// Validates passed arguments and creates a local & remote machines between
/// which data will be copied.
fn runner() -> Result<(), CrustError> {
    let raw_args = parser::RawArgs::parse();

    let args = parser::ValidatedArgs::new(raw_args)?;

    let src_machine: Box<dyn Machine> = if args.src_hostname.is_none() {
        Box::new(LocalMachine::new())
    } else {
        Box::new(RemoteMachine::new(
            args.src_username.unwrap(),
            args.src_hostname.unwrap(),
            args.password.clone(),
            args.pkey.clone(),
            args.port,
        ))
    };

    let dst_machine: Box<dyn Machine> = if args.dst_hostname.is_none() {
        Box::new(LocalMachine::new())
    } else {
        Box::new(RemoteMachine::new(
            args.dst_username.unwrap(),
            args.dst_hostname.unwrap(),
            args.password.clone(),
            args.pkey.clone(),
            args.port,
        ))
    };

    let split_size = match args.chunk_size {
        Some(v) => v,
        None => {
            let threads = args.threads.unwrap() as u64;
            let cmd = format!("du -b {}", args.src_path);

            let total_size: u64 = src_machine
                .exec(cmd.as_str())?
                .split_whitespace()
                .next()
                .expect("File size can not be determined")
                .parse()
                .expect("File size can not be converted to an integer");

            let chunk_size = total_size / threads;
            if total_size % threads == 0 {
                chunk_size
            } else {
                chunk_size + 1
            }
        }
    };

    let parts = src_machine
        .split(split_size, args.src_path.as_str())
        .unwrap();
    copy(parts, &src_machine, &dst_machine, args.verbose);
    dst_machine.merge(args.dst_path.as_str())?;

    Ok(())
}

/// Copied data with threads to destination.
/// Coping process is always invoked on localmachine (only order of arguments
/// could changed). After start every thread, waits till end of each to finish
/// method. In case of error, destructor guarantees cleanup for temporary files.
#[allow(clippy::borrowed_box)]
fn copy(
    chunks: Vec<PathBuf>,
    src_machine: &Box<dyn Machine>,
    dst_machine: &Box<dyn Machine>,
    verbose: bool,
) {
    let status_results = Arc::new(Mutex::new(Vec::new()));

    let handles: Vec<_> = chunks
        .into_iter()
        .map(|file_path| {
            let status_results = Arc::clone(&status_results);

            let to: String;
            let from: String;
            match src_machine.mtype() {
                MachineType::LocalMachine => {
                    //upload
                    to = format!("{}:{}", dst_machine.ssh_address(), dst_machine.get_tmpdir());
                    from = file_path.clone().to_string_lossy().to_string();
                }
                MachineType::RemoteMachine => {
                    // download
                    to = dst_machine.get_tmpdir();
                    from = format!(
                        "{}:{}",
                        src_machine.ssh_address(),
                        file_path.clone().to_string_lossy()
                    );
                }
                MachineType::AbstractMachine => {
                    panic!("TODO: to implement");
                }
            }

            thread::spawn(move || {
                let (stdout_verbose, stderr_verbose) = match verbose {
                    true => (
                        std::process::Stdio::inherit(),
                        std::process::Stdio::inherit(),
                    ),
                    false => (std::process::Stdio::null(), std::process::Stdio::null()),
                };
                let status = Command::new("scp")
                    .arg(from)
                    .arg(to)
                    .stdout(stdout_verbose)
                    .stderr(stderr_verbose)
                    .status()
                    .expect("failed to run scp");

                let mut results = status_results.lock().unwrap();
                results.push((file_path, status));
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

fn main() {
    let result = runner();
    handle_result::<(), DefaultExitHandler>(result);
}
