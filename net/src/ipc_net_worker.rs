use holochain_net_ipc::{
    ipc_client::IpcClient,
    socket::{IpcSocket, MockIpcSocket, ZmqIpcSocket},
    util::get_millis,
};

use holochain_net_connection::{
    net_connection::{NetConnection, NetConnectionRelay, NetHandler, NetWorker},
    protocol::{JsonString, Protocol},
    NetResult,
};

use std::{fmt::Debug, sync::mpsc};

use serde_json;

pub struct IpcNetWorker {
    handler: NetHandler,
    ipc_relay: NetConnectionRelay,
    ipc_relay_receiver: mpsc::Receiver<Protocol>,

    is_ready: bool,

    state: String,
    id: String,
    bindings: Vec<String>,

    last_state_millis: f64,
}

impl NetWorker for IpcNetWorker {
    fn destroy(self: Box<Self>) -> NetResult<()> {
        self.ipc_relay.destroy()?;
        Ok(())
    }

    fn receive(&mut self, data: Protocol) -> NetResult<()> {
        self.ipc_relay.send(data)?;
        Ok(())
    }

    fn tick(&mut self) -> NetResult<bool> {
        let mut did_something = false;

        if &self.state != "ready" {
            self.priv_check_init()?;
        }

        if self.ipc_relay.tick()? {
            did_something = true;
        }

        if let Ok(data) = self.ipc_relay_receiver.try_recv() {
            did_something = true;

            if data.is_json() {
                let json: serde_json::Value = serde_json::from_str(&data.as_json_string())?;

                println!("@@@@ got {}", serde_json::to_string_pretty(&json)?);

                if json.is_object() && json["method"].is_string() {
                    if json["method"] == "state" {
                        self.priv_handle_state(json)?;
                    } else if json["method"] == "defaultConfig" {
                        self.priv_handle_default_config(json)?;
                    }
                }
            }

            (self.handler)(Ok(data))?;
        }

        Ok(did_something)
    }
}

impl IpcNetWorker {
    pub fn new(handler: NetHandler, config: &JsonString) -> NetResult<Self> {
        let config: serde_json::Value = config.into();

        let (ipc_sender, ipc_relay_receiver) = mpsc::channel::<Protocol>();

        let ipc_relay = NetConnectionRelay::new(
            Box::new(move |r| {
                ipc_sender.send(r?)?;
                Ok(())
            }),
            Box::new(move |h| {
                let mut socket: Box<IpcSocket> = match config["socketType"].as_str().unwrap() {
                    "mock" => MockIpcSocket::new()?,
                    "zmq" => ZmqIpcSocket::new()?,
                    _ => bail!("unexpected socketType: {}", config["socketType"]),
                };
                socket.connect(config["ipcUri"].as_str().unwrap())?;

                Ok(Box::new(IpcClient::new(h, socket)?))
            }),
        )?;

        Ok(IpcNetWorker {
            handler,
            ipc_relay,
            ipc_relay_receiver,

            is_ready: false,

            state: "undefined".to_string(),
            id: "undefined".to_string(),
            bindings: Vec::new(),

            last_state_millis: 0.0_f64,
        })
    }

    // -- private -- //

    fn priv_check_init(&mut self) -> NetResult<()> {
        let now = get_millis();

        if now - self.last_state_millis > 500.0 {
            self.ipc_relay.send(
                json!({
                "method": "requestState",
            }).to_string()
                    .into(),
            )?;
            self.last_state_millis = now;
        }

        Ok(())
    }

    fn priv_handle_state(&mut self, json: serde_json::Value) -> NetResult<()> {
        self.state = match json["state"].as_str() {
            Some(s) => s.to_string(),
            None => "undefined".to_string(),
        };

        self.id = match json["id"].as_str() {
            Some(id) => id.to_string(),
            None => "undefined".to_string(),
        };

        self.bindings = match json["bindings"].as_array() {
            Some(arr) => arr
                .iter()
                .map(|i| match i.as_str() {
                    Some(b) => b.to_string(),
                    None => "undefined".to_string(),
                })
                .collect(),
            None => Vec::new(),
        };

        if &self.state == "need_config" {
            self.ipc_relay.send(
                json!({
                "method": "requestDefaultConfig",
            }).to_string()
                    .into(),
            )?;
        }

        if !self.is_ready && &self.state == "ready" {
            (self.handler)(Ok(Protocol::P2pReady))?;
        }

        println!(
            "GOT STATE UPDATE: state: {}, id: {}, bindings: {:?}",
            self.state, self.id, self.bindings
        );

        Ok(())
    }

    fn priv_handle_default_config(&mut self, json: serde_json::Value) -> NetResult<()> {
        if &self.state == "need_config" {
            self.ipc_relay.send(
                json!({
                "method": "setConfig",
                "config": json["config"],
            }).to_string()
                    .into(),
            )?;
        }

        Ok(())
    }
}
