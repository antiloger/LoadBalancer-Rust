use std::sync::{
    atomic::{
        AtomicU64,
        Ordering::{self, SeqCst},
    },
    Arc,
};

use tokio::sync::RwLock;

pub struct Server {
    url: String,
    alive: Arc<RwLock<bool>>,
    reverse_proxy: String,
}

impl Server {
    pub fn new(url: String, setalive: bool, rp: String) -> Self {
        Server {
            url,
            alive: Arc::new(RwLock::new(false)),
            reverse_proxy: rp,
        }
    }

    pub async fn set_alive(&self, alive: bool) {
        let mut al = self.alive.write().await;
        *al = alive
    }

    pub async fn is_alive(&self) -> bool {
        self.alive.read().await.clone()
    }
}

pub struct ServersPool {
    servers: Arc<RwLock<Vec<Server>>>,
    current: Arc<AtomicU64>,
}

impl ServersPool {
    pub fn new() -> Self {
        ServersPool {
            servers: Arc::new(RwLock::new(Vec::new())),
            current: Arc::new(AtomicU64::new(0)),
        }
    }

    // TODO: need to handle error -> if the server count 0 or error
    pub async fn server_count(&self) -> usize {
        let ser = self.servers.read().await;
        ser.len()
    }

    pub async fn next_server_idx(&self) -> usize {
        let next = self.current.fetch_add(1, SeqCst) as usize;
        next % self.server_count().await
    }

    pub async fn is_alive(&self, index: usize) -> bool {
        let srv = self.servers.read().await;
        srv[index].is_alive().await
    }

    pub async fn get_nextpeer(&self) -> Option<usize> {
        let nxt = self.next_server_idx().await;
        let l = self.server_count().await + nxt;

        for i in nxt..l {
            let idx = i % self.server_count().await;
            if self.is_alive(idx).await {
                if i != nxt {
                    self.current.store(idx as u64, Ordering::SeqCst);
                }

                return Some(idx);
            }
        }

        None
    }
}
