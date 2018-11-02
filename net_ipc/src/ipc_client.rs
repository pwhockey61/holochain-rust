use socket::IpcSocket;
use util::get_millis;

use std::{thread, time};

use holochain_net_connection::{
    net_connection::{NetHandler, NetWorker},
    protocol::{NamedBinaryData, PingData, PongData, Protocol},
    NetResult,
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
        self.priv_send(&data)
    }

    fn tick(&mut self) -> NetResult<bool> {
        let mut did_something = false;

        if let Some(msg) = self.priv_proc_message()? {
            did_something = true;

            match msg {
                Protocol::Ping(ref p) => {
                    self.priv_send(&Protocol::Pong(PongData {
                        orig: p.sent,
                        recv: get_millis(),
                    }))?;
                }
                _ => (),
            }

            (self.handler)(Ok(msg))?;
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
    /// establish a new ipc connection
    /// for now, the api simplicity is worth blocking the thread on connection
    pub fn new(handler: NetHandler, mut socket: Box<IpcSocket>) -> NetResult<Self> {
        let start = get_millis();
        let mut backoff = 1_u64;

        loop {
            // wait for any message from server to indicate connect success
            if socket.poll(0)? {
                break;
            }

            if get_millis() - start > 3000.0 {
                bail!("connection init timeout");
            }

            let data = Protocol::Ping(PingData { sent: get_millis() });
            let data: NamedBinaryData = data.into();
            socket.send(&[
                &[0x24, 0x24, 0x24, 0x24],
                &[],
                &b"ping".to_vec(),
                &data.data,
            ])?;

            backoff *= 2;
            if backoff > 500 {
                backoff = 500;
            }

            thread::sleep(time::Duration::from_millis(backoff));
        }

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
        if res.len() != 4 {
            bail!("bad msg len: {}", res.len());
        }

        // we got a message, update our timeout counter
        self.last_recv_millis = get_millis();

        let msg = NamedBinaryData {
            name: res[2].to_vec(),
            data: res[3].to_vec(),
        };

        let msg: Protocol = msg.into();

        Ok(Some(msg))
    }

    /// Send a heartbeat message to the ipc server.
    fn priv_ping(&mut self) -> NetResult<()> {
        self.priv_send(&Protocol::Ping(PingData { sent: get_millis() }))?;
        //self.priv_send(&"{\"method\": \"requestState\"}".into())?;
        /*
        self.priv_send(&"{\"test\": \"hello\"}".into())?;
        self.priv_send(&Protocol::NamedBinary(NamedBinaryData {
            name: b"test".to_vec(),
            data: b"a".to_vec(),
        }))?;
        */
        Ok(())
    }

    /// send a raw message to the ipc server
    fn priv_send(&mut self, data: &Protocol) -> NetResult<()> {
        let data: NamedBinaryData = data.into();

        // with two zmq "ROUTER" sockets, one side must have a well-known id
        // for the holochain ipc protocol, the server is always 4 0x24 bytes
        self.socket
            .send(&[&[0x24, 0x24, 0x24, 0x24], &[], &data.name, &data.data])?;

        // sent message, update our ping timer
        self.last_send_millis = get_millis();

        Ok(())
    }
}
