use std::fs;

use serde::{Deserialize, Serialize};

use crate::{
    lberror::LBError,
    rrlb::{Server, ServersPool},
};

#[derive(Serialize, Deserialize, Debug)]
struct ServerInfo {
    LbAddr: String,
    LbPort: u16,
    Servers: Vec<Backends>,
}

impl ServerInfo {
    fn convert(&self) -> ServersPool {
        let mut server_vec = Vec::new();
        for i in self.Servers.iter() {
            server_vec.push(Server::new(i.Addr.clone(), i.Port, i.Alive));
        }

        ServersPool::new(server_vec, self.LbAddr.clone(), self.LbPort)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Backends {
    Addr: String,
    Port: u16,
    Alive: bool,
    LogFile: String,
}

pub fn read_servers() -> ServersPool {
    let content = fs::read_to_string("servers.json").expect("can not read servers.json");
    let data = serde_json::from_str::<ServerInfo>(&content).expect("can not convert json file");

    let d = data.convert();
    d
}
