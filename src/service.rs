use crate::message::{Channel, Message};
use crate::utils::{execute, sleep_ms};
use anyhow::Result;
use std::sync::mpsc;
use tracing::{debug, error, info};
use crate::message::Message::LoginDone;

pub struct Service {
    pub channel: Channel,
    pub self_loop: Channel,
}

unsafe impl Send for Service {}

impl Service {
    async fn handle_message(&mut self, msg: Message) -> Result<bool> {
        info!("service handle msg: {:?}", msg);
        match msg {
            Message::ApiClientConnect(_) => {}
            Message::LoginDone(m) => { self.channel.tx.send(LoginDone(m)).unwrap(); }
        }
        Ok(false)
    }

    pub fn new(channel: Channel) -> Self {
        let (channel_loop_tx, channel_loop_rx) = mpsc::channel();
        let self_loop = Channel {
            tx: channel_loop_tx,
            rx: channel_loop_rx,
        };
        Self {
            channel,
            self_loop,
        }
    }

    pub async fn run(&mut self) {
        loop {
            sleep_ms(10).await;
            {
                let r = self
                    .channel
                    .rx
                    .try_recv()
                    .map(|msg| self.handle_message(msg));
                if let Ok(r) = r {
                    match r.await {
                        Ok(will_break) => {
                            if will_break {
                                break;
                            }
                        }
                        Err(e) => {
                            error!("service run error: {}", e);
                            break;
                        }
                    };
                }
            }
            {
                let r = self
                    .self_loop
                    .rx
                    .try_recv()
                    .map(|msg| self.handle_message(msg));
                if let Ok(r) = r {
                    match r.await {
                        Ok(will_break) => {
                            if will_break {
                                break;
                            }
                        }
                        Err(e) => {
                            error!("service loop run error: {}", e);
                            break;
                        }
                    };
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        info!("[thread-{:?}] service stopped", std::thread::current().id());
        #[cfg(target_arch = "wasm32")]
        info!("service stopped");
    }

    pub fn start(channel: Channel) {
        debug!("starting service...");
        // execute(async move {
        execute(async move {
            run_service(channel).await;
        });
        debug!("service started");
    }
}

async fn run_service(channel: Channel) {
    let mut s = Service::new(channel);
    #[cfg(not(target_arch = "wasm32"))]
    info!("[thread-{:?}] service starts", std::thread::current().id());
    #[cfg(target_arch = "wasm32")]
    info!("service starts");
    s.run().await
}
