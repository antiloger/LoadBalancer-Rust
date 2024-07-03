use std::{marker, net::SocketAddr, sync::Arc};

use files::read_servers;
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{
    body::Bytes, client::conn::http1::Builder, server::conn::http1, service::service_fn, Request,
    Response,
};
use hyper_util::rt::TokioIo;
use lberror::LBError;
use rrlb::ServersPool;
use tokio::net::{TcpListener, TcpStream};
mod files;
mod lberror;
mod rrlb;

#[tokio::main]
async fn main() {
    env_logger::init();
    let pool = read_servers();
    server(pool).await.unwrap();
}

// async fn proxy_handler(req: Request<impl hyper::body::Body>) -> Result<Response<Body>, hyper::Error> {
//     let uri = req.uri().path_and_query()
// }
async fn get_server(serpool: &Arc<ServersPool>) -> Result<(TcpStream, usize), LBError> {
    let count = serpool.server_count().await;
    for _ in 0..count {
        let peer_id = match serpool.get_nextpeer().await {
            Some(p) => p,
            None => return Err(LBError::NoPeerError),
        };

        let peer_addr = serpool.get_peer_addr(peer_id).await;

        match TcpStream::connect(peer_addr).await {
            Ok(s) => return Ok((s, peer_id)),
            Err(e) => {
                serpool.set_server_status(peer_id, false).await;
                log::error!(
                    "server {peer_id} dosen't response | turn off the server : \n{:?}",
                    e
                );
                continue;
            }
        };
    }

    log::error!("CRITICAL: No Peer Error");
    Err(LBError::NoPeerError)
}

async fn proxy_handler(
    mut req: Request<hyper::body::Incoming>,
    serverpool: Arc<ServersPool>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, LBError> {
    let (stream, peer_id) = match get_server(&serverpool).await {
        Ok(s) => s,
        Err(e) => return Err(e),
    };

    let (addr, port) = serverpool.get_peer_addr(peer_id).await;

    let uri_str = format!(
        "http://{}:{}{}",
        addr,
        port,
        req.uri().path_and_query().map(|x| x.as_str()).unwrap()
    );
    *req.uri_mut() = uri_str.parse().unwrap();

    //TODO: add correct server

    let io = TokioIo::new(stream);

    let (mut sender, conn) = Builder::new()
        .preserve_header_case(true)
        .title_case_headers(true)
        .handshake(io)
        .await
        .unwrap();

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            log::error!("error on connection: {:?}", err);
        }
    });

    let resp = sender.send_request(req).await.unwrap();
    log::info!("server:{peer_id} | {uri_str}");
    Ok(resp.map(|b| b.boxed()))
}

async fn server(serverpool: ServersPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let addr2 = serverpool.get_addr();

    let listener = TcpListener::bind(addr).await?;

    let server_status = Arc::new(serverpool);
    log::info!("Server Start on {:?}", addr2);

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        let state = server_status.clone();

        tokio::task::spawn(async move {
            let service = service_fn(move |req| {
                let s = state.clone();
                async move { proxy_handler(req, s).await }
            });
            if let Err(err) = http1::Builder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .serve_connection(io, service)
                .with_upgrades()
                .await
            {
                eprint!("error serving connection: {:?}", err);
                log::error!("error serving connection {:?}", err);
            }
        });
    }
}
