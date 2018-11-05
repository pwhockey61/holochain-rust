use holochain_net_connection::{
    net_connection::{NetConnection, NetHandler},
    net_connection_thread::NetConnectionThread,
    protocol::{JsonString, Protocol},
    NetResult,
};

use super::ipc_net_worker::IpcNetWorker;

use serde_json;

#[derive(Debug)]
pub struct P2pNetwork {
    con: NetConnectionThread,
}

impl NetConnection for P2pNetwork {
    fn send(&mut self, data: Protocol) -> NetResult<()> {
        self.con.send(data)
    }
}

impl P2pNetwork {
    pub fn new(handler: NetHandler, config: &JsonString) -> NetResult<Self> {
        let config: serde_json::Value = config.into();
        if &config["backend"] == "ipc" {
            return Ok(P2pNetwork {
                con: NetConnectionThread::new(
                    handler,
                    Box::new(move |h| {
                        Ok(Box::new(IpcNetWorker::new(
                            h,
                            &(config["config"].to_string().into()),
                        )?))
                    }),
                )?,
            });
        }
        bail!("unknown p2p_network backend: {}", config["backend"]);
    }

    pub fn destroy(self) -> NetResult<()> {
        self.con.destroy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_fail_bad_backend_type() {
        let res = P2pNetwork::new(
            Box::new(|_r| Ok(())),
            &JsonString(
                json!({
            "backend": "bad"
        }).to_string(),
            ),
        ).expect_err("should have thrown")
            .to_string();
        assert!(res.contains("backend: \"bad\""), "res: {}", res);
    }
}
