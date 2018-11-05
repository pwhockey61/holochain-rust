extern crate holochain_net;
extern crate holochain_net_connection;
#[macro_use]
extern crate serde_json;

use holochain_net_connection::{
    net_connection_thread::NetConnectionThread, protocol::Protocol, NetResult,
};

use holochain_net::{
    ipc_net_worker::IpcNetWorker,
    p2p_network::P2pNetwork,
};

use std::sync::mpsc;

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn usage() {
    println!("Usage: test_bin_ipc <ipc_uri>");
    std::process::exit(1);
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn exec() -> NetResult<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        usage();
    }

    let ipc_uri = args[1].clone();

    if ipc_uri == "" {
        usage();
    }

    println!("testing against uri: {}", ipc_uri);

    let (sender, receiver) = mpsc::channel::<Protocol>();

    let mut con = P2pNetwork::new(
        Box::new(move |r| {
            sender.send(r?)?;
            Ok(())
        }),
        &json!({
            "backend": "ipc",
            "config": {
                "socketType": "zmq",
                "ipcUri": ipc_uri,
            }
        }).into(),
    )?;

    loop {
        let z = receiver.recv()?;

        println!("got: {:?}", z);

        if let Protocol::P2pReady = z {
            println!("p2p ready!!");
            break;
        }
    }

    con.destroy()?;

    Ok(())
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn main() {
    exec().unwrap();
}
