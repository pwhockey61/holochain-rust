extern crate holochain_net;
extern crate failure;

use holochain_net::{
    p2p_network::P2pNetwork,
    ipc::P2pIpcZmq,
};
use failure::Error;

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn usage() {
    println!("Usage: holochain_test_ipc_network_suite ipc://<ipc socket path here>");
    std::process::exit(1);
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        usage();
    }

    let ipc_uri = &args[1];

    if ipc_uri == "" {
        usage();
    }

    println!("try connect to {:?}", ipc_uri);
    run_suite(ipc_uri).unwrap();
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn run_suite(ipc_uri: &str) -> Result<(), Error> {
    let mut con = P2pIpcZmq::new()?;
    con.connect(ipc_uri)?;

    println!("## check state");
    println!("-> {:?}", con.get_state()?);

    Ok(())
}
