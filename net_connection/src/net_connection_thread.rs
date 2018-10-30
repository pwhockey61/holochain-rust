use crate::net_connection::{
    JsonString,
    NetConnection,
    NetHandler,
    NetResult,
    NetWorkerFactory,
};

use std::sync::{
    Arc,
    atomic::{
        AtomicBool,
        Ordering
    },
    mpsc,
};

pub struct NetConnectionThread {
    keep_running: Arc<AtomicBool>,
    send_channel: mpsc::Sender<JsonString>,
    thread: std::thread::JoinHandle<()>,
}

impl NetConnection for NetConnectionThread {
    fn send(&mut self, data: JsonString) -> NetResult<()> {
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

    pub fn new(
        handler: NetHandler,
        worker_factory: Box<NetWorkerFactory>
    ) -> NetResult<Self> {
        let keep_running = Arc::new(AtomicBool::new(true));
        let keep_running2 = keep_running.clone();

        let (sender, receiver) = mpsc::channel();
        Ok(NetConnectionThread {
            keep_running,
            send_channel: sender,
            thread: std::thread::spawn(move || {
                let mut us = 100_u64;
                let mut worker = match worker_factory.new(handler) {
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
                                Err(e) => panic!(e)
                            };
                        },
                        Err(_) => (),
                    };

                    match worker.tick() {
                        Ok(b) => {
                            if b {
                                did_something = true;
                            }
                        },
                        Err(e) => panic!(e),
                    };

                    if did_something {
                        us = 100_u64;
                    } else {
                        us *= 2_u64;
                        if us > 10_000_u64 {
                            us = 10_000_u64;
                        }
                    }

                    std::thread::sleep(std::time::Duration::from_micros(us));
                }
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::net_connection::{
        NetWorker,
    };

    struct DefWorker;

    impl NetWorker for DefWorker {
    }

    struct DefWorkerFactory;

    impl NetWorkerFactory for DefWorkerFactory {
        fn new (
            &self,
            _handler: NetHandler,
        ) -> NetResult<Box<NetWorker>> {
            Ok(Box::new(DefWorker))
        }
    }

    #[test]
    fn it_can_defaults() {
        let factory = DefWorkerFactory;
        let mut con = NetConnectionThread::new(Box::new(move |_r| {
            Ok(())
        }), Box::new(factory)).unwrap();

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

        fn receive(&mut self, data: JsonString) -> NetResult<()> {
            (self.handler)(Ok(data))
        }
    }

    struct WorkerFactory;

    impl NetWorkerFactory for WorkerFactory {
        fn new(
            &self,
            handler: NetHandler,
        ) -> NetResult<Box<NetWorker>> {
            Ok(Box::new(Worker {
                handler,
            }))
        }
    }

    #[test]
    fn it_invokes_connection_thread() {
        let (sender, receiver) = std::sync::mpsc::channel();

        let factory = WorkerFactory;
        let mut con = NetConnectionThread::new(Box::new(move |r| {
            sender.send(r?)?;
            Ok(())
        }), Box::new(factory)).unwrap();

        con.send("test".into()).unwrap();

        let res = receiver.recv().unwrap();

        assert_eq!("test".to_string(), res);

        con.destroy().unwrap();
    }

    #[test]
    fn it_can_tick() {
        let (sender, receiver) = std::sync::mpsc::channel();

        let factory = WorkerFactory;
        let con = NetConnectionThread::new(Box::new(move |r| {
            sender.send(r?)?;
            Ok(())
        }), Box::new(factory)).unwrap();

        let res = receiver.recv().unwrap();

        assert_eq!("tick".to_string(), res);

        con.destroy().unwrap();
    }
}
