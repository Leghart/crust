use std::cell::RefCell;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use clap::Parser;
use text_colorizer::Colorize;

pub mod connection;
pub mod error;
pub mod exec;
pub mod interfaces;
pub mod logger;
pub mod machine;
pub mod utils;

#[cfg(test)]
pub mod mocks;
pub mod parser;
pub mod scp;

use connection::manager::MachinesManager;
use connection::parser::BaseConnArgs;
use error::{handle_result, CrustError, DefaultExitHandler};
use interfaces::parser::Validation;
use interfaces::response::CrustResult;
use logger::Logger;
use machine::local::LocalMachine;
use machine::remote::RemoteMachine;
use machine::Machine;
use parser::{AppArgs, Operation};
use scp::scp;
use utils::shell_manager::ShellManager;

static LOGGER: Logger = Logger;

/// Function to dynamic creating a machine with (or without) alias.
/// Tries to get machine from manager at first. If alias does not exists
/// in manager, try to create a new one (only if additional data was passed -
/// if not return Error - early access)
fn get_or_create_remote_machine(
    args: impl BaseConnArgs,
    manager: &mut MachinesManager,
) -> Result<Rc<RefCell<Box<dyn Machine>>>, CrustError> {
    let machine = match &args.alias() {
        None => {
            log::trace!("Creating remote machine (without alias)");
            let (user, host) = args.split_addr();
            RemoteMachine::get_or_create(
                user,
                host,
                args.password().map(|s| s.to_owned()),
                args.pkey().map(|pb| pb.to_owned()),
                args.port().unwrap(),
                None,
                manager,
            )
        }
        Some(alias) => {
            log::trace!("Passed machine alias '{alias}'. Trying to get from manager...");
            match RemoteMachine::get(alias, manager) {
                Some(m) => m,
                None => {
                    log::trace!(
                        "No machine with the given alias ({alias}) was found in the manager. Trying to create a new one..."
                    );
                    if args.addr().is_some() && (args.password().is_some() || args.pkey().is_some())
                    {
                        log::trace!(
                            "Required args to create machine are found - creating a new one"
                        );
                        let (user, host) = args.split_addr();
                        RemoteMachine::get_or_create(
                            user,
                            host,
                            args.password().map(|s| s.to_owned()),
                            args.pkey().map(|pb| pb.to_owned()),
                            args.port().unwrap(),
                            args.alias().map(|s| s.to_owned()),
                            manager,
                        )
                    } else {
                        return Err(CrustError {
                            code: error::ExitCode::Internal,
                            message: format!("There is no registered machine with alias '{alias}'"),
                        });
                    }
                }
            }
        }
    };
    Ok(machine)
}

/// Entrypoint for CLI invoke.
fn single_run(
    mut args: AppArgs,
    manager_opt: Option<&mut MachinesManager>,
) -> Result<CrustResult, CrustError> {
    args.validate()?;
    log::trace!("Validated args: {:#?}", args);

    let mut default_manager = MachinesManager::default();
    let manager = match manager_opt {
        Some(man) => man,
        None => &mut default_manager,
    };

    let operation = args.get_operation();

    // Set up a new session when operation was not passed
    if operation.is_none() {
        log::info!("Starting a new session");
        return Ok(CrustResult::default());
    }

    let result = match operation.unwrap() {
        Operation::Exec(exec_args) => {
            let machine = match &exec_args.remote {
                Some(_args) => get_or_create_remote_machine(_args.clone(), manager)?,
                None => LocalMachine::get_or_create(manager),
            };

            let cmd = exec_args.cmd.as_ref().unwrap().join(" ");
            match exec_args.rt {
                true => machine.borrow().exec_rt(&cmd, exec_args.merge)?,
                false => machine.borrow().exec(&cmd)?,
            }
        }
        Operation::Scp(scp_args) => {
            let src_machine = match &scp_args.src.remote_params {
                None => LocalMachine::get_or_create(manager),
                Some(_args) => get_or_create_remote_machine(_args.clone(), manager)?,
            };

            let dst_machine = match &scp_args.dst.remote_params {
                None => LocalMachine::get_or_create(manager),
                Some(_args) => get_or_create_remote_machine(_args.clone(), manager)?,
            };

            scp(
                &src_machine,
                &dst_machine,
                PathBuf::from(&scp_args.src.path_from),
                PathBuf::from(&scp_args.dst.path_to),
                scp_args.progress,
                scp_args.threads,
            )?
        }
    };

    Ok(result)
}

/// Read data from standard input (used in manual invoke).
fn read_stdin() -> String {
    print!("\n[q to exit]>> ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("error: unable to read user input");
    input
}

/// Read data from specific FIFO pipe (only in shell invoke).
/// Potential race condition - if it is a first invoke, shell script
/// has to create tmp_dir and pipe, but in the meanwhile crust will try
/// to get data from pipe. To protect against panic, method wait for
/// `timeout=5` seconds to create a fifo.
fn read_fifo() -> String {
    let mut input = String::new();
    log::warn!("waiting for fifo...");

    let fifo = format!("/tmp/tmp_crust_{}/fifo", std::process::id());

    let timeout = 5;
    let start = std::time::Instant::now();
    while !std::path::Path::new(&fifo).exists()
        && start + Duration::from_secs(timeout) >= std::time::Instant::now()
    {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let file = std::fs::File::open(fifo)
        .unwrap_or_else(|_| panic!("FIFO was not created during {timeout}s"));
    let mut reader = std::io::BufReader::new(file);

    reader.read_line(&mut input).unwrap();
    input
}

/// Allows to run in background mode (store connections).
/// Supports two ways of invoke: via command line or bash script
/// manager (should be used in external scripts).
fn multi_runs(args: AppArgs) {
    let mut manager = MachinesManager::default();
    let mut curr_args = args.clone();
    let read_input = match ShellManager::is_background_mode() {
        true => read_fifo,
        false => read_stdin,
    };
    loop {
        let result = single_run(curr_args, Some(&mut manager));

        match result {
            Ok(cr) => match cr.is_success() {
                true => println!("{}", cr.stdout().green()),
                false => println!("{}", cr.stderr().red()),
            },
            Err(e) => log::error!("{e}"),
        };

        let input = read_input();

        if input == "q\n" {
            break;
        }

        let path_exe = std::env::current_exe().expect("No executable path");
        let mut base_input: Vec<&str> = vec![path_exe.as_path().to_str().unwrap()];
        let mut iter = input.split(' ').map(|x| x.trim()).collect::<Vec<&str>>();
        base_input.append(&mut iter);
        log::debug!("user cmd: {:?}", base_input);
        curr_args = AppArgs::parse_from(base_input);

        logger::init(&curr_args.verbose.log_level_filter()); //TODO: for background invoke from shell, it's first initialization
    }
}

pub fn main() {
    let args = parser::AppArgs::parse();

    if !(ShellManager::is_background_mode() && ShellManager::is_shell_invoke()) {
        logger::init(&args.verbose.log_level_filter());
    }

    match args.background {
        false => {
            let result = single_run(args, None);
            handle_result::<DefaultExitHandler>(result);
        }
        true => multi_runs(args),
    }
}
