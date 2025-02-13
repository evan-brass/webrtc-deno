#[cfg(test)]
mod periodic_timer_test;

use tokio::sync::{mpsc, Mutex};
use std::time::Duration;

use std::sync::Arc;

use async_trait::async_trait;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TimerIdRefresh {
    Alloc,
    Perms,
}

impl Default for TimerIdRefresh {
    fn default() -> Self {
        TimerIdRefresh::Alloc
    }
}

// PeriodicTimerTimeoutHandler is a handler called on timeout
#[async_trait(?Send)]
pub trait PeriodicTimerTimeoutHandler {
    async fn on_timeout(&mut self, id: TimerIdRefresh);
}

// PeriodicTimer is a periodic timer
#[derive(Default)]
pub struct PeriodicTimer {
    id: TimerIdRefresh,
    interval: Duration,
    close_tx: Mutex<Option<mpsc::Sender<()>>>,
}

impl PeriodicTimer {
    // create a new timer
    pub fn new(id: TimerIdRefresh, interval: Duration) -> Self {
        PeriodicTimer {
            id,
            interval,
            close_tx: Mutex::new(None),
        }
    }

    // Start starts the timer.
    pub async fn start<T: 'static + PeriodicTimerTimeoutHandler + std::marker::Send>(
        &self,
        timeout_handler: Arc<Mutex<T>>,
    ) -> bool {
        // this is a noop if the timer is always running
        {
            let close_tx = self.close_tx.lock().await;
            if close_tx.is_some() {
                return false;
            }
        }

        let (close_tx, mut close_rx) = mpsc::channel(1);
        let interval = self.interval;
        let id = self.id;

        wasm_bindgen_futures::spawn_local(async move {
            loop {
                let timer = deno_net::sleep(interval);
                tokio::pin!(timer);

                tokio::select! {
                    _ = timer.as_mut() => {
                        let mut handler = timeout_handler.lock().await;
                        handler.on_timeout(id).await;
                    }
                    _ = close_rx.recv() => break,
                }
            }
        });

        {
            let mut close = self.close_tx.lock().await;
            *close = Some(close_tx);
        }

        true
    }

    // Stop stops the timer.
    pub async fn stop(&self) {
        let mut close_tx = self.close_tx.lock().await;
        close_tx.take();
    }

    // is_running tests if the timer is running.
    // Debug purpose only
    pub async fn is_running(&self) -> bool {
        let close_tx = self.close_tx.lock().await;
        close_tx.is_some()
    }
}
