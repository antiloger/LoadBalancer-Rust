use std::sync::{
    atomic::{
        AtomicU64,
        Ordering::{self, SeqCst},
    },
    Arc,
};

use tokio::sync::RwLock;

#[derive(Debug)]
pub struct Server {
    addr: String,
    port: u16,
    alive: Arc<RwLock<bool>>,
}

impl Server {
    pub fn new(addr: String, port: u16, setalive: bool) -> Self {
        Server {
            addr,
            port,
            alive: Arc::new(RwLock::new(setalive)),
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

#[derive(Debug)]
pub struct ServersPool {
    Addr: String,
    Port: u16,
    servers: Arc<RwLock<Vec<Server>>>,
    current: Arc<AtomicU64>,
}

impl ServersPool {
    pub fn new(servers: Vec<Server>, addr: String, port: u16) -> Self {
        ServersPool {
            Addr: addr,
            Port: port,
            servers: Arc::new(RwLock::new(servers)),
            current: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn get_addr(&self) -> (String, u16) {
        (self.Addr.clone(), self.Port)
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
            log::info!("|| server {idx} is not alive ||");
        }

        None
    }

    pub async fn get_peer_addr(&self, idx: usize) -> (String, u16) {
        let sev = self.servers.read().await;
        (sev[idx].addr.clone(), sev[idx].port)
    }

    pub async fn set_server_status(&self, idx: usize, status: bool) {
        let sev = self.servers.read().await;
        sev[idx].set_alive(status).await
    }
}
