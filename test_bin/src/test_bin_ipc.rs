extern crate holochain_net_connection;
extern crate holochain_net_ipc;

use holochain_net_connection::{
    NetResult,
    protocol::{
        Protocol,
    },
    net_connection::{
        NetConnection,
    },
    net_connection_thread::{
        NetConnectionThread,
    },
};

use holochain_net_ipc::{
    socket::{
        IpcSocket,
        ZmqIpcSocket,
    },
    ipc_client_2::{
        IpcClient
    },
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

    let mut con = NetConnectionThread::new(Box::new(move |r| {
        sender.send(r?)?;
        Ok(())
    }), Box::new(move |h| {
        let mut socket = ZmqIpcSocket::new()?;
        socket.connect(&ipc_uri)?;

        Ok(Box::new(IpcClient::new(h, socket)?))
    }))?;

    con.send("{\"frm_test_ipc\":\"hello\"}".into())?;

    loop {
        let z = receiver.recv()?;

        println!("got: {:?}", z);
    }

    con.destroy()?;

    Ok(())
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn main() {
    exec().unwrap();
}
