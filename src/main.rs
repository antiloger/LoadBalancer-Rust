use std::net::SocketAddr;

use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{
    body::Bytes, client::conn::http1::Builder, server::conn::http1, service::service_fn, Request,
    Response,
};
use hyper_util::rt::TokioIo;
use tokio::net::{TcpListener, TcpStream};
mod rrlb;

#[tokio::main]
async fn main() {
    server().await.unwrap();
}

// async fn proxy_handler(req: Request<impl hyper::body::Body>) -> Result<Response<Body>, hyper::Error> {
//     let uri = req.uri().path_and_query()
// }

async fn proxy_handler(
    mut req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let uri_str = format!(
        "http://127.0.0.1:8080{}",
        req.uri().path_and_query().map(|x| x.as_str()).unwrap()
    );
    *req.uri_mut() = uri_str.parse().unwrap();

    //TODO: add correct server

    let stream = TcpStream::connect(("127.0.0.1", 8080)).await.unwrap();
    let io = TokioIo::new(stream);

    let (mut sender, conn) = Builder::new()
        .preserve_header_case(true)
        .title_case_headers(true)
        .handshake(io)
        .await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection Failed: {:?}", err);
        }
    });

    let resp = sender.send_request(req).await?;

    Ok(resp.map(|b| b.boxed()))
}

async fn server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .serve_connection(io, service_fn(proxy_handler))
                .with_upgrades()
                .await
            {
                eprint!("error serving connection: {:?}", err)
            }
        });
    }
}
