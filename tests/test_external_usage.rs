extern crate crust;

use crust::connection::manager::{MachinesManager, MachinesManagerMethods};
use crust::machine::local::LocalMachine;
use crust::machine::remote::RemoteMachine;
use crust::machine::MachineID;

#[test]
fn test_extern_usage_background_connections() {
    let mut manager = MachinesManager::default();

    let _ = LocalMachine::get_or_create(&mut manager);
    let id: MachineID;
    {
        let remote = RemoteMachine::get_or_create(
            String::from("test_user"),
            String::from("10.10.10.10"),
            Some(String::from("1234")),
            None,
            22,
            None,
            &mut manager,
        );
        id = remote.borrow().get_id().clone();

        assert_eq!(
            remote.borrow().exec("pwd").unwrap().stdout(),
            "/home/test_user\n"
        );
        assert_eq!(manager.size(), 2);
    }
    assert_eq!(manager.size(), 2);

    let remote_ref = manager.get_machine(&id).unwrap().borrow();
    assert_eq!(
        remote_ref.exec("pwd").unwrap().stdout(),
        "/home/test_user\n"
    );
}

#[test]
fn test_extern_usage_background_connections_use_alias() {
    let mut manager = MachinesManager::default();

    let _ = RemoteMachine::get_or_create(
        String::from("test_user"),
        String::from("10.10.10.10"),
        Some(String::from("1234")),
        None,
        22,
        Some(String::from("alias")),
        &mut manager,
    );

    let remote_ref = RemoteMachine::get("alias", &mut manager).unwrap();
    assert_eq!(
        remote_ref.borrow().exec("pwd").unwrap().stdout(),
        "/home/test_user\n"
    );
}
