use holochain_net_ipc::{
    ipc_client::IpcClient,
    socket::{IpcSocket, MockIpcSocket, ZmqIpcSocket},
    util::get_millis,
};

use holochain_net_connection::{
    net_connection::{NetConnection, NetConnectionRelay, NetHandler, NetWorker},
    protocol::{JsonString, Protocol},
    protocol_wrapper::{
        ProtocolWrapper,
        StateData,
        ConfigData,
    },
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

            let wrap = ProtocolWrapper::from(&data);

            match wrap {
                ProtocolWrapper::State(s) => {
                    self.priv_handle_state(s)?;
                },
                ProtocolWrapper::DefaultConfig(c) => {
                    self.priv_handle_default_config(c)?;
                },
                _ => (),
            };

            (self.handler)(Ok(data))?;

            if !self.is_ready && &self.state == "ready" {
                self.is_ready = true;
                (self.handler)(Ok(Protocol::P2pReady))?;
            }
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

            last_state_millis: 0.0_f64,
        })
    }

    // -- private -- //

    fn priv_check_init(&mut self) -> NetResult<()> {
        let now = get_millis();

        if now - self.last_state_millis > 500.0 {
            self.ipc_relay.send(ProtocolWrapper::RequestState.into())?;
            self.last_state_millis = now;
        }

        Ok(())
    }

    fn priv_handle_state(&mut self, state: StateData) -> NetResult<()> {
        self.state = state.state;

        if &self.state == "need_config" {
            self.ipc_relay.send(ProtocolWrapper::RequestDefaultConfig.into())?;
        }

        println!(
            "GOT STATE UPDATE: state: {}, id: {}, bindings: {:?}",
            self.state, state.id, state.bindings
        );

        Ok(())
    }

    fn priv_handle_default_config(&mut self, config: ConfigData) -> NetResult<()> {
        if &self.state == "need_config" {
            self.ipc_relay.send(ProtocolWrapper::SetConfig(ConfigData {
                config: config.config,
            }).into())?;
        }

        Ok(())
    }
}
