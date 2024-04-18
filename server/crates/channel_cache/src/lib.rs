use std::{borrow::Cow, sync::Arc};

use anyhow::{bail, Result};
use futures::Future;
use tokio::{
    sync::{mpsc, oneshot, Mutex},
    task::JoinHandle,
};
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct ChannelCacheTask<S, R> {
    #[allow(dead_code)]
    name: Cow<'static, str>,
    tx: mpsc::Sender<Option<ChannelValue<S, R>>>,
    join_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl<S, R> ChannelCacheTask<S, R>
where
    S: Send + 'static,
    R: Send + 'static,
{
    pub fn new<F, Fut>(
        name: Cow<'static, str>,
        max_size: usize,
        trigger_ms: usize,
        mut trigger: F,
    ) -> Self
    where
        F: FnMut(Vec<S>) -> Fut + Send + 'static,
        Fut: Future<Output = Vec<R>> + Send + 'static,
    {
        let (tx, mut rx) = mpsc::channel::<Option<ChannelValue<S, R>>>(max_size);
        let task_name = name.clone();
        let join_handle = tokio::spawn(async move {
            use std::time::Instant;

            let mut senders = Vec::with_capacity(max_size);
            let mut buffer = Vec::with_capacity(max_size);
            let mut last_time = Instant::now();
            let mut run = true;
            while run {
                let now = Instant::now();
                // Try to receive a new value without blocking
                match rx.try_recv() {
                    Ok(cvalue) => {
                        // If there is a new value, add it to the buffer and the senders
                        match cvalue {
                            Some(cv) => {
                                if buffer.is_empty() {
                                    last_time = now.clone();
                                }
                                senders.push(cv.sender);
                                buffer.push(cv.value);
                            }
                            None => run = false,
                        }
                    }
                    Err(err) => match err {
                        mpsc::error::TryRecvError::Empty => (),
                        mpsc::error::TryRecvError::Disconnected => {
                            info!("channel sender disconnected.");
                            run = false;
                        }
                    },
                }
                // Check if the buffer is full or the time has elapsed
                if !buffer.is_empty() {
                    if buffer.len() >= max_size || (now - last_time).as_millis() >= trigger_ms as _
                    {
                        if buffer.len() >= max_size {
                            info!("[{task_name}] Task trigger: Buffer reaches maximum size.");
                        } else {
                            info!("[{task_name}] Task trigger: {trigger_ms}ms interval.");
                        }
                        let last = Instant::now();
                        let mut returned = trigger(buffer).await;
                        info!("[{task_name}] Task end: Usage time {:?}", Instant::now() - last);
                        returned.reverse();
                        let mut i = returned.len();
                        for v in returned {
                            i -= 1;
                            let sender = senders.swap_remove(i);

                            if let Err(_) = sender.send(v) {
                                // send value to who send to task.
                                warn!("receiver dropped.");
                            }
                        }
                        if senders.len() > 0 {
                            panic!("no clear all sender...")
                        }
                        buffer = Vec::with_capacity(max_size);
                        senders = Vec::with_capacity(max_size);
                        last_time = Instant::now();
                    }
                } else {
                    tokio::time::sleep(std::time::Duration::from_nanos(1)).await;  // dont let runtime stuck in this task.
                }
            }
        });

        ChannelCacheTask {
            name,
            tx,
            join_handle: Arc::new(Mutex::new(Some(join_handle))),
        }
    }

    pub async fn send(&self, value: S) -> Result<R> {
        let (cv, rx) = ChannelValue::new(value);
        if let Err(_err) = self.tx.send(Some(cv)).await {
            bail!("Can't send value. mpsc receiver dropped.")
        }
        match rx.await {
            Ok(returned) => Ok(returned),
            Err(_) => bail!("Can't receive value. oneshot sender dropped."),
        }
    }

    #[allow(dead_code)]
    pub async fn stop(&self) {
        let handle = self.join_handle.lock().await.take();
        if let Some(handle) = handle {
            if let Err(err) = handle.await {
                warn!("stop task occurs error: {}", err);
            }
        }
    }
}

#[derive(Debug)]
struct ChannelValue<S, R> {
    value: S,
    sender: oneshot::Sender<R>,
}

impl<S, R> ChannelValue<S, R> {
    pub fn new(value: S) -> (ChannelValue<S, R>, oneshot::Receiver<R>) {
        let (sender, rx) = oneshot::channel::<R>();
        let this = Self { value, sender };
        (this, rx)
    }
}
