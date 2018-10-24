//! Concrete struct implementation of P2pNetwork Trait
//! Allows holochain connections to separate p2p networking processes
//! over IPC (zeromq) socket channels

use failure::Error;

use p2p_network::{
    P2pNetwork,
    ApiFnJson,
    ApiFnBin,
};

use holochain_net_ipc::{
    socket::{
        IpcSocket,
        MockIpcSocket,
    },
};

pub mod ipc;

use ipc::ipc::Ipc;

type IpcResult<T> = Result<T, Error>;

pub struct P2pIpc<S: IpcSocket> {
    cli: Ipc<S>,
}

impl<S: IpcSocket> P2pIpc<S> {
    pub fn new() -> IpcResult<Self> {
        let cli: Ipc<S> = Ipc::new()?;

        Ok(P2pIpc {
            cli
        })
    }

    pub fn connect(&mut self, uri: &str) -> IpcResult<()> {
        self.cli.connect(uri)?;
        Ok(())
    }
}

impl<S: IpcSocket> P2pNetwork for P2pIpc<S> {
    /// execute the next test stub json handler
    fn exec_raw_json(&mut self, input: &str, _cb: Option<ApiFnJson>) -> Result<String, Error> {
        println!("raw_json: {:?}", input);
        Ok("undefined".to_string())
    }

    /// execute the next test stub binary handler
    fn exec_raw_bin(&mut self, input: &[u8], _cb: Option<ApiFnBin>) -> Result<Vec<u8>, Error> {
        println!("raw_bin: {:?}", input);
        Ok(vec![])
    }
}

impl P2pIpc<MockIpcSocket> {
    pub fn priv_test_inject_pong(&mut self) {
        self.cli.priv_test_inject_pong();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_create() {
        let mut p: P2pIpc<MockIpcSocket> = P2pIpc::new().unwrap();
        p.priv_test_inject_pong();
        p.connect("ipc://test-socket.socket").unwrap();
        p.get_default_config().unwrap();
    }
}
