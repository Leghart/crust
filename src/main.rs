use std::path::PathBuf;

use clap::Parser;
use connection::manager::MachinesManagerMethods;

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

use crate::scp::parser::ValidatedArgs;
use crate::scp::scp;

use machine::base::Machine;

use machine::local::LocalMachine;
use machine::remote::RemoteMachine;
use parser::Operation;

use error::{handle_result, CrustError, DefaultExitHandler};

use crate::interfaces::parser::Validation;
static LOGGER: logger::Logger = logger::Logger;

/// Entrypoint for crust.
fn runner() -> Result<(), CrustError> {
    let mut args = parser::AppArgs::parse();
    logger::init(&args.verbose.log_level_filter())?;

    args.validate()?;
    log::trace!("Validated args: {:?}", args);

    let mut manager = connection::manager::MachinesManager::new();

    match args.get_operation() {
        Operation::Exec(exec_args) => {
            let machine: Box<dyn Machine> = match &exec_args.remote {
                Some(_args) => {
                    let (user, host) = _args.split_addr();
                    Box::new(RemoteMachine::new(
                        user.to_string(),
                        host.to_string(),
                        _args.password_to.clone(),
                        _args.pkey_to.clone(),
                        _args.port_to.unwrap(),
                        &mut manager,
                    ))
                }
                None => Box::new(LocalMachine::new(&mut manager)),
            };
            let r = machine.exec(&exec_args.cmd)?;
            println!("{}", r);
        }
        Operation::Scp(scp_args) => {
            let args = ValidatedArgs::validate_and_create(scp_args.clone())?;

            let src_machine: Box<dyn Machine> = if args.hostname_from.is_none() {
                Box::new(LocalMachine::new(&mut manager))
            } else {
                Box::new(RemoteMachine::new(
                    args.username_from.unwrap(),
                    args.hostname_from.unwrap(),
                    args.password_from.clone(),
                    args.pkey_from.clone(),
                    args.port_from.unwrap(),
                    &mut manager,
                ))
            };

            let dst_machine: Box<dyn Machine> = if args.hostname_to.is_none() {
                Box::new(LocalMachine::new(&mut manager))
            } else {
                Box::new(RemoteMachine::new(
                    args.username_to.unwrap(),
                    args.hostname_to.unwrap(),
                    args.password_to.clone(),
                    args.pkey_to.clone(),
                    args.port_to.unwrap(),
                    &mut manager,
                ))
            };

            let src_id = src_machine.get_id();
            let dst_id = dst_machine.get_id();
            let _from = PathBuf::from(args.path_from);
            let _to = PathBuf::from(args.path_to);
            let mut src_ref = manager.get_machine(src_id).unwrap().borrow_mut();
            let mut dst_ref = manager.get_machine(dst_id).unwrap().borrow_mut();

            scp(&mut src_ref, &mut dst_ref, _from, _to, args.progress)?;
        }
    }

    Ok(())
}

fn main() {
    let result = runner();
    handle_result::<(), DefaultExitHandler>(result);
}
