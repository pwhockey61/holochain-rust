use rmp_serde;
use serde;

use errors::*;
use socket::IpcSocket;
use util::*;

use holochain_net_connection::{
    NetResult,
    protocol::{
        PingData,
        Protocol,
    },
    net_connection::{
        NetHandler,
        NetWorker,
    },
};

pub struct IpcClient {
    handler: NetHandler,
    socket: Box<IpcSocket>,
    last_recv_millis: f64,
    last_send_millis: f64,
}

impl NetWorker for IpcClient {
    fn destroy(self: Box<Self>) -> NetResult<()> {
        self.socket.close()?;
        Ok(())
    }

    fn receive(&mut self, data: Protocol) -> NetResult<()> {
        println!("ipc client recv: {}", data.as_json_string());
        Ok(())
    }

    fn tick(&mut self) -> NetResult<bool> {
        let mut did_something = false;

        if let Some(msg) = self.priv_proc_message()? {
            println!("got message!: {:?}", msg);
            did_something = true;
        }

        let now = get_millis();

        if now - self.last_recv_millis > 2000.0 {
            bail!("ipc connection timeout");
        }

        if now - self.last_send_millis > 500.0 {
            self.priv_ping()?;
            did_something = true;
        }

        Ok(did_something)
    }
}

impl IpcClient {
    pub fn new(handler: NetHandler, socket: Box<IpcSocket>) -> NetResult<Self> {
        Ok(Self {
            handler,
            socket,
            last_recv_millis: get_millis(),
            last_send_millis: 0.0,
        })
    }

    // -- private -- //

    fn priv_proc_message(&mut self) -> NetResult<Option<Protocol>> {
        if !self.socket.poll(0)? {
            return Ok(None);
        }

        // we have data, let's fetch it
        let res = self.socket.recv()?;
        if res.len() != 3 {
            bail!("bad msg len: {}", res.len());
        }

        // we got a message, update our timeout counter
        self.last_recv_millis = get_millis();

        let msg: Protocol = rmp_serde::from_slice(&res[2])?;
        Ok(Some(msg))
    }

    /// Send a heartbeat message to the ipc server.
    fn priv_ping(&mut self) -> NetResult<()> {
        self.priv_send(&Protocol::Ping(PingData {
            sent: get_millis()
        }))?;
        self.priv_send(&"hello".into())?;
        Ok(())
    }

    /// send a raw message to the ipc server
    fn priv_send(&mut self, data: &Protocol) -> NetResult<()> {
        let data = rmp_serde::to_vec(data)?;

        // with two zmq "ROUTER" sockets, one side must have a well-known id
        // for the holochain ipc protocol, the server is always 4 0x24 bytes
        self.socket.send(&[&[0x24, 0x24, 0x24, 0x24], &[], &data])?;

        // sent message, update our ping timer
        self.last_send_millis = get_millis();

        Ok(())
    }
}
