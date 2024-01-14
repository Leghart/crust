use clap::Parser;

pub mod connection;
pub mod error;
pub mod exec;
pub mod interfaces;
pub mod machine;
pub mod parser;
pub mod tscp;

use machine::base::Machine;

use machine::local::LocalMachine;
use machine::remote::RemoteMachine;
use parser::Operation;
use tscp::{download, upload, Tscp};

use crate::tscp::parser::ValidatedArgs;

use error::{handle_result, CrustError, DefaultExitHandler};

use crate::interfaces::parser::Validation;
use crate::interfaces::tmpdir::TemporaryDirectory;

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
        Operation::Tscp(ref tscp_args) => {
            let args = ValidatedArgs::validate_and_create(tscp_args.clone())?;

            match args.src_hostname.is_none() {
                true => {
                    // upload
                    let mut src_machine = LocalMachine::new();
                    let mut dst_machine = RemoteMachine::new(
                        args.dst_username.clone().unwrap(),
                        args.dst_hostname.clone().unwrap(),
                        args.password.clone(),
                        args.pkey.clone(),
                        args.port,
                    );
                    src_machine.create_tmpdir();
                    dst_machine.create_tmpdir();

                    let split_size = args.get_split_size(&src_machine);
                    let parts = src_machine.split(split_size, args.src_path.as_str())?;

                    upload::upload(parts, &src_machine, &dst_machine)?;
                    dst_machine.merge(args.dst_path.as_str())?;
                }
                false => {
                    // download
                    let mut dst_machine = LocalMachine::new();
                    let mut src_machine = RemoteMachine::new(
                        args.src_username.clone().unwrap(),
                        args.src_hostname.clone().unwrap(),
                        args.password.clone(),
                        args.pkey.clone(),
                        args.port,
                    );
                    src_machine.create_tmpdir();
                    dst_machine.create_tmpdir();

                    let split_size = args.get_split_size(&src_machine);
                    let parts = src_machine.split(split_size, args.src_path.as_str())?;

                    download::download(parts, &src_machine, &dst_machine)?;
                    dst_machine.merge(args.dst_path.as_str())?;
                }
            }
        }
    }
    Ok(())
}

fn main() {
    let result = runner();
    handle_result::<(), DefaultExitHandler>(result);
}
