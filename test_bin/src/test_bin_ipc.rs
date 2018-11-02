extern crate holochain_net;
extern crate holochain_net_connection;

use holochain_net_connection::{
    net_connection_thread::NetConnectionThread, protocol::Protocol, NetResult,
};

use holochain_net::ipc_net_worker::{IpcNetWorker, IpcNetWorkerConfig, IpcSocketType};

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

    /*
    let mut con = NetConnectionThread::new(
        Box::new(move |r| {
            sender.send(r?)?;
            Ok(())
        }),
        Box::new(move |h| {
            let mut socket = ZmqIpcSocket::new()?;
            socket.connect(&ipc_uri)?;

            Ok(Box::new(IpcClient::new(h, socket)?))
        }),
    )?;
    */

    #[derive(Debug)]
    struct Config {
        socket_type: IpcSocketType,
        uri: String,
    }

    impl IpcNetWorkerConfig for Config {
        fn get_socket_type<'a>(&'a self) -> &'a IpcSocketType {
            &self.socket_type
        }

        fn get_ipc_uri<'a>(&'a self) -> &'a str {
            &self.uri
        }
    }

    let mut con = NetConnectionThread::new(
        Box::new(move |r| {
            sender.send(r?)?;
            Ok(())
        }),
        Box::new(move |h| {
            Ok(Box::new(IpcNetWorker::new(
                h,
                Box::new(Config {
                    socket_type: IpcSocketType::Zmq,
                    uri: ipc_uri.clone(),
                }),
            )?))
        }),
    )?;

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
