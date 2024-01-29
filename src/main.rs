use std::path::PathBuf;

use clap::Parser;
use connection::manager::MachinesManagerMethods;

pub mod connection;
pub mod error;
pub mod exec;
pub mod interfaces;
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

/// Entrypoint for crust.
fn runner() -> Result<(), CrustError> {
    let mut args = parser::AppArgs::parse();
    args.validate()?;

    let mut manager = connection::manager::MachinesManager::new();

    match args.get_operation() {
        Operation::Exec(exec_args) => {
            let machine: Box<dyn Machine> = match &exec_args.remote {
                Some(_args) => Box::new(RemoteMachine::new(
                    _args.user.clone().unwrap(),
                    _args.host.clone().unwrap(),
                    _args.password.clone(),
                    _args.pkey.clone(),
                    _args.port,
                    &mut manager,
                )),
                None => Box::new(LocalMachine::new(&mut manager)),
            };

            let r = machine.exec(&exec_args.cmd)?;
            println!("{}", r);
        }
        Operation::Scp(scp_args) => {
            let args = ValidatedArgs::validate_and_create(scp_args.clone())?;

            let src_machine: Box<dyn Machine> = if args.src_hostname.is_none() {
                Box::new(LocalMachine::new(&mut manager))
            } else {
                Box::new(RemoteMachine::new(
                    args.src_username.unwrap(),
                    args.src_hostname.unwrap(),
                    args.password.clone(),
                    args.pkey.clone(),
                    args.port,
                    &mut manager,
                ))
            };

            let dst_machine: Box<dyn Machine> = if args.dst_hostname.is_none() {
                Box::new(LocalMachine::new(&mut manager))
            } else {
                Box::new(RemoteMachine::new(
                    args.dst_username.unwrap(),
                    args.dst_hostname.unwrap(),
                    args.password.clone(),
                    args.pkey.clone(),
                    args.port,
                    &mut manager,
                ))
            };
            let src_id = src_machine.get_id();
            let dst_id = dst_machine.get_id();
            let _from = PathBuf::from(args.src_path);
            let _to = PathBuf::from(args.dst_path);

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
