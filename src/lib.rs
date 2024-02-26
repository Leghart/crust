use std::io::{self, Write};
use std::path::PathBuf;

use clap::Parser;
use text_colorizer::Colorize;

pub mod connection;
pub mod error;
pub mod exec;
pub mod interfaces;
pub mod logger;
pub mod machine;
#[cfg(test)]
pub mod mocks;
pub mod parser;
pub mod scp;

use connection::manager::MachinesManager;
use error::{handle_result, CrustError, DefaultExitHandler};
use interfaces::parser::Validation;
use interfaces::response::CrustResult;
use logger::Logger;
use machine::local::LocalMachine;
use machine::remote::RemoteMachine;
use parser::{AppArgs, Operation};
use scp::parser::ValidatedArgs;
use scp::scp;

static LOGGER: Logger = Logger;

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

    let result = match args.get_operation() {
        Operation::Exec(exec_args) => {
            let machine = match &exec_args.remote {
                Some(_args) => {
                    let (user, host) = _args.split_addr();
                    RemoteMachine::get_or_create(
                        user,
                        host,
                        _args.password_to.clone(),
                        _args.pkey_to.clone(),
                        _args.port_to.unwrap(),
                        manager,
                    )
                }
                None => LocalMachine::get_or_create(manager),
            };

            let cmd = exec_args.cmd.as_ref().unwrap().join(" ");
            match exec_args.rt {
                true => machine.borrow().exec_rt(&cmd, exec_args.merge)?,
                false => machine.borrow().exec(&cmd)?,
            }
        }
        Operation::Scp(scp_args) => {
            let args = ValidatedArgs::validate_and_create(scp_args.clone())?;

            let src_machine = if args.hostname_from.is_none() {
                LocalMachine::get_or_create(manager)
            } else {
                RemoteMachine::get_or_create(
                    args.username_from.unwrap(),
                    args.hostname_from.unwrap(),
                    args.password_from.clone(),
                    args.pkey_from.clone(),
                    args.port_from.unwrap(),
                    manager,
                )
            };

            let dst_machine = if args.hostname_to.is_none() {
                LocalMachine::get_or_create(manager)
            } else {
                RemoteMachine::get_or_create(
                    args.username_to.unwrap(),
                    args.hostname_to.unwrap(),
                    args.password_to.clone(),
                    args.pkey_to.clone(),
                    args.port_to.unwrap(),
                    manager,
                )
            };

            scp(
                &src_machine,
                &dst_machine,
                PathBuf::from(args.path_from),
                PathBuf::from(args.path_to),
                args.progress,
            )?
        }
    };

    Ok(result)
}

fn multi_runs(args: AppArgs) {
    let mut manager = MachinesManager::default();
    let mut curr_args = args.clone();
    loop {
        let result = single_run(curr_args, Some(&mut manager));

        match result {
            Ok(cr) => match cr.is_success() {
                true => println!("{}", cr.stdout().green()),
                false => println!("{}", cr.stderr().red()),
            },
            Err(e) => log::error!("{e}"),
        };

        print!("\n[q to exit]>> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("error: unable to read user input");

        if input == "q\n" {
            break;
        }

        let path_exe = std::env::current_exe().expect("No executable path");
        let mut base_input: Vec<&str> = vec![path_exe.as_path().to_str().unwrap()];
        let mut iter = input.split(' ').map(|x| x.trim()).collect::<Vec<&str>>();
        base_input.append(&mut iter);
        log::debug!("user cmd: {:?}", base_input);
        curr_args = AppArgs::parse_from(base_input);
    }
}

pub fn main() {
    let args = parser::AppArgs::parse();
    logger::init(&args.verbose.log_level_filter()).expect("Logger error");

    log::debug!("Background mode: {}", args.background);
    match args.background {
        false => {
            let result = single_run(args, None);
            handle_result::<DefaultExitHandler>(result);
        }
        true => multi_runs(args),
    }
}
