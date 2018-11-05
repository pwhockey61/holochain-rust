extern crate holochain_net;
extern crate holochain_net_connection;
#[macro_use]
extern crate serde_json;

use holochain_net_connection::{
    net_connection::NetConnection,
    net_connection_thread::NetConnectionThread, protocol::Protocol, protocol_wrapper::{ProtocolWrapper, ConnectData, SendData,}, NetResult,
};

use holochain_net::{ipc_net_worker::IpcNetWorker, p2p_network::P2pNetwork};

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

    let mut id = "".to_string();
    let mut addr = "".to_string();

    loop {
        let z = receiver.recv()?;

        let wrap = ProtocolWrapper::from(&z);

        match wrap {
            ProtocolWrapper::State(s) => {
                id = s.id;
                if s.bindings.len() > 0 {
                    addr = s.bindings[0].clone();
                }
            },
            _ => (),
        }

        if let Protocol::P2pReady = z {
            println!("p2p ready!!");
            break;
        }
    }

    println!("id: {}, addr: {}", id, addr);

    con.send(ProtocolWrapper::Connect(ConnectData {
        address: addr.clone(),
    }).into())?;

    loop {
        let z = receiver.recv()?;

        match ProtocolWrapper::from(&z) {
            ProtocolWrapper::PeerConnected(p) => {
                println!("got peer connected: {}", p.id);
                break;
            },
            _ => (),
        }

        println!("got: {:?}", z);
    }

    con.send(ProtocolWrapper::SendMessage(SendData {
        msg_id: "unique-id".to_string(),
        to_address: id.clone(),
        data: json!("test data"),
    }).into())?;

    let mut handleData: ProtocolWrapper;

    loop {
        let z = receiver.recv()?;

        match ProtocolWrapper::from(&z) {
            ProtocolWrapper::HandleSend(m) => {
                handleData = ProtocolWrapper::HandleSend(m);
                break;
            },
            _ => (),
        }

        println!("got: {:?}", z);
    }

    println!("got handleSend: {:?}", handleData);

    con.destroy()?;

    Ok(())
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn main() {
    exec().unwrap();
}
