use super::NetResult;
use protocol::Protocol;

pub type NetHandler = Box<FnMut(NetResult<Protocol>) -> NetResult<()> + Send>;

pub trait NetConnection {
    fn send(&mut self, data: Protocol) -> NetResult<()>;
}

pub trait NetWorker {
    fn destroy(self: Box<Self>) -> NetResult<()> {
        Ok(())
    }

    fn receive(&mut self, _data: Protocol) -> NetResult<()> {
        Ok(())
    }

    fn tick(&mut self) -> NetResult<bool> {
        Ok(false)
    }
}

pub type NetWorkerFactory = Box<FnMut(NetHandler) -> NetResult<Box<NetWorker>> + Send>;

pub struct NetConnectionRelay {
    worker: Box<NetWorker>,
}

impl NetConnection for NetConnectionRelay {
    fn send(&mut self, data: Protocol) -> NetResult<()> {
        self.worker.receive(data)?;
        Ok(())
    }
}

impl NetConnectionRelay {
    pub fn destroy(self) -> NetResult<()> {
        self.worker.destroy()?;
        Ok(())
    }

    pub fn tick(&mut self) -> NetResult<bool> {
        self.worker.tick()
    }

    pub fn new(handler: NetHandler, mut worker_factory: NetWorkerFactory) -> NetResult<Self> {
        Ok(NetConnectionRelay {
            worker: worker_factory(handler)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::mpsc;

    struct DefWorker;

    impl NetWorker for DefWorker {}

    #[test]
    fn it_can_defaults() {
        let mut con =
            NetConnectionRelay::new(Box::new(move |_r| Ok(())), Box::new(|_h| Ok(Box::new(DefWorker)))).unwrap();

        con.send("test".into()).unwrap();
        con.tick().unwrap();
        con.destroy().unwrap();
    }

    struct Worker {
        handler: NetHandler,
    }

    impl NetWorker for Worker {
        fn tick(&mut self) -> NetResult<bool> {
            (self.handler)(Ok("tick".into()))?;
            Ok(true)
        }

        fn receive(&mut self, data: Protocol) -> NetResult<()> {
            (self.handler)(Ok(data))
        }
    }

    #[test]
    fn it_invokes_connection_relay() {
        let (sender, receiver) = mpsc::channel();

        let mut con = NetConnectionRelay::new(
            Box::new(move |r| {
                sender.send(r?)?;
                Ok(())
            }),
            Box::new(|h| {
                Ok(Box::new(Worker { handler: h }))
            }),
        ).unwrap();

        con.send("test".into()).unwrap();

        let res = receiver.recv().unwrap();

        assert_eq!(&b"test".to_vec(), res.as_json());

        con.destroy().unwrap();
    }

    #[test]
    fn it_can_tick() {
        let (sender, receiver) = mpsc::channel();

        let mut con = NetConnectionRelay::new(
            Box::new(move |r| {
                sender.send(r?)?;
                Ok(())
            }),
            Box::new(|h| {
                Ok(Box::new(Worker { handler: h }))
            }),
        ).unwrap();

        con.tick().unwrap();

        let res = receiver.recv().unwrap();

        assert_eq!(&b"tick".to_vec(), res.as_json());

        con.destroy().unwrap();
    }
}
