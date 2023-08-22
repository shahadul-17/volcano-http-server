#[path = "utilities/http_utilities.rs"]
mod http_utilities;

use hyper::{
    http::Error as HyperHttpError,
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use std::{
    convert::Infallible,
    net::SocketAddr,
    sync::atomic::{AtomicU64, Ordering},
};

static HTTP_REQUEST_COUNT: AtomicU64 = AtomicU64::new(1_u64);

async fn handle_request_async(
    request_id: u64,
    remote_address: SocketAddr,
    request: Request<Body>,
) -> Result<Response<Body>, HyperHttpError> {
    let http_request =
        http_utilities::serialize_http_request_async(request_id, remote_address, request).await;
    let data = http_request.to_string();
    let response = Response::builder()
        .status(200)
        .header("X-Powered-By", "Node.js")
        .header("Content-Type", "application/json")
        // .body(Body::from("Hello World from RUST HTTP server..!!"));
        .body(Body::from(data));

    return response;
}

pub async fn start_async(host: String, port: String) {
    let socket_address_option = http_utilities::create_socket_address(host, port);

    if socket_address_option.is_none() {
        eprintln!("An error occurred while creating socket address.");

        return;
    }

    let service_function = |socket: &AddrStream| {
        let remote_address = socket.remote_addr();

        async move {
            Ok::<_, Infallible>(service_fn(move |request| async move {
                let request_id = HTTP_REQUEST_COUNT.fetch_add(1_u64, Ordering::SeqCst);

                return handle_request_async(request_id, remote_address, request).await;
            }))
        }
    };

    let make_service = make_service_fn(service_function);
    let socket_address = socket_address_option.unwrap();
    let server = Server::bind(&socket_address).serve(make_service);

    if let Err(error) = server.await {
        eprintln!("An unexpected error occurred: {error}");
    }
}
