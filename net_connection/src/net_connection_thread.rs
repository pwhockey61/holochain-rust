use super::NetResult;

use super::{
    net_connection::{NetConnection, NetHandler, NetWorkerFactory},
    protocol::Protocol,
};

use std::{thread, time};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};

pub struct NetConnectionThread {
    keep_running: Arc<AtomicBool>,
    send_channel: mpsc::Sender<Protocol>,
    thread: thread::JoinHandle<()>,
}

impl NetConnection for NetConnectionThread {
    fn send(&mut self, data: Protocol) -> NetResult<()> {
        self.send_channel.send(data)?;
        Ok(())
    }
}

impl NetConnectionThread {
    pub fn destroy(self) -> NetResult<()> {
        self.keep_running.store(false, Ordering::Relaxed);
        match self.thread.join() {
            Ok(_) => Ok(()),
            Err(_) => {
                bail!("NetConnectionThread failed to join on destroy");
            }
        }
    }

    pub fn new(handler: NetHandler, mut worker_factory: NetWorkerFactory) -> NetResult<Self> {
        let keep_running = Arc::new(AtomicBool::new(true));
        let keep_running2 = keep_running.clone();

        let (sender, receiver) = mpsc::channel();
        Ok(NetConnectionThread {
            keep_running,
            send_channel: sender,
            thread: thread::spawn(move || {
                let mut us = 100_u64;
                let mut worker = match worker_factory(handler) {
                    Ok(w) => w,
                    Err(e) => panic!(e),
                };

                while keep_running2.load(Ordering::Relaxed) {
                    let mut did_something = false;

                    match receiver.try_recv() {
                        Ok(data) => {
                            did_something = true;
                            match worker.receive(data) {
                                Ok(_) => (),
                                Err(e) => panic!(e),
                            };
                        }
                        Err(_) => (),
                    };

                    match worker.tick() {
                        Ok(b) => {
                            if b {
                                did_something = true;
                            }
                        }
                        Err(e) => panic!("{:?}", e),
                    };

                    if did_something {
                        us = 100_u64;
                    } else {
                        us *= 2_u64;
                        if us > 10_000_u64 {
                            us = 10_000_u64;
                        }
                    }

                    thread::sleep(time::Duration::from_micros(us));
                }
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::net_connection::NetWorker;

    struct DefWorker;

    impl NetWorker for DefWorker {}

    #[test]
    fn it_can_defaults() {
        let mut con = NetConnectionThread::new(
            Box::new(move |_r| Ok(())),
            Box::new(|_h| Ok(Box::new(DefWorker))),
        ).unwrap();

        con.send("test".into()).unwrap();
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
    fn it_invokes_connection_thread() {
        let (sender, receiver) = mpsc::channel();

        let mut con = NetConnectionThread::new(
            Box::new(move |r| {
                sender.send(r?)?;
                Ok(())
            }),
            Box::new(|h| Ok(Box::new(Worker { handler: h }))),
        ).unwrap();

        con.send("test".into()).unwrap();

        let res = receiver.recv().unwrap();

        assert_eq!("test".to_string(), res.as_json_string());

        con.destroy().unwrap();
    }

    #[test]
    fn it_can_tick() {
        let (sender, receiver) = mpsc::channel();

        let con = NetConnectionThread::new(
            Box::new(move |r| {
                sender.send(r?)?;
                Ok(())
            }),
            Box::new(|h| Ok(Box::new(Worker { handler: h }))),
        ).unwrap();

        let res = receiver.recv().unwrap();

        assert_eq!("tick".to_string(), res.as_json_string());

        con.destroy().unwrap();
    }
}
