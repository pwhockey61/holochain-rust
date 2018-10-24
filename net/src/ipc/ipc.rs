//! zmq net_ipc wrapper implementing call handlers

use failure::Error;

use holochain_net_ipc::{
    ipc_client::IpcClient,
    socket::{
        IpcSocket,
        MockIpcSocket,
    },
};

type IpcResult<T> = Result<T, Error>;

pub struct Ipc<S: IpcSocket> {
    cli: IpcClient<S>,
}

impl<S: IpcSocket> Ipc<S> {
    pub fn new() -> IpcResult<Self> {
        let cli: IpcClient<S> = IpcClient::new()?;

        Ok(Ipc {
            cli
        })
    }

    pub fn connect(&mut self, uri: &str) -> IpcResult<()> {
        self.cli.connect(uri)?;
        Ok(())
    }
}

impl Ipc<MockIpcSocket> {
    pub fn priv_test_inject_pong(&mut self) {
        self.cli.priv_test_inject_pong();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_create() {
        let mut p: Ipc<MockIpcSocket> = Ipc::new().unwrap();
        p.priv_test_inject_pong();
        p.connect("ipc://test-socket.socket").unwrap();
    }
}
