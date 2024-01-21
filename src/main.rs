use clap::Parser;

pub mod connection;
pub mod error;
pub mod exec;
pub mod interfaces;
pub mod machine;
pub mod parser;

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

    match args.get_operation() {
        Operation::Exec(exec_args) => {
            let machine: Box<dyn Machine> = match &exec_args.remote {
                Some(_args) => Box::new(RemoteMachine::new(
                    _args.user.clone().unwrap(),
                    _args.host.clone().unwrap(),
                    _args.password.clone(),
                    _args.pkey.clone(),
                    _args.port,
                )),
                None => Box::new(LocalMachine::new()),
            };

            let r = machine.exec(&exec_args.cmd)?;
            println!("{}", r);
        }
    }
    Ok(())
}

fn main() {
    let result = runner();
    handle_result::<(), DefaultExitHandler>(result);
}
