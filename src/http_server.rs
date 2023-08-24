#[path = "utilities/http_utilities.rs"]
mod http_utilities;

use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use std::{
    convert::Infallible,
    net::SocketAddr,
    sync::atomic::{AtomicU64, Ordering},
};
use tokio::sync::watch;
use watch::Receiver;

use self::http_utilities::{SerializableHttpRequest, SerializableHttpResponse};
use crate::ipc_handler;

static HTTP_REQUEST_COUNT: AtomicU64 = AtomicU64::new(1_u64);

async fn get_parent_process_response_async(
    http_request: &SerializableHttpRequest,
    receiver: &Receiver<(u64, String)>,
) -> Response<Body> {
    // converts http request to JSON...
    let http_request_as_json = http_request.to_string();
    let request_id = http_request.request_id;

    // writes the http request data to the standard output as JSON...
    ipc_handler::write_line(&http_request_as_json);

    let receiver = receiver.clone();
    // reads the specified line...
    let line_read = ipc_handler::read_line_async(request_id, receiver).await;
    let serializable_http_response_option = SerializableHttpResponse::from(line_read);

    if serializable_http_response_option.is_none() {
        return Response::builder()
            .status(500)
            .body(Body::from("ERROR"))
            .unwrap();
    }

    // preparing the response...
    let response = serializable_http_response_option.unwrap().to_response();

    return response;
}

async fn handle_request_async(
    request_id: u64,
    remote_address: SocketAddr,
    request: Request<Body>,
    receiver: &Receiver<(u64, String)>,
) -> Response<Body> {
    let receiver = receiver.clone();
    let http_request =
        http_utilities::serialize_http_request_async(request_id, remote_address, request).await;
    let response = get_parent_process_response_async(&http_request, &receiver).await;

    return response;
}

pub async fn start_async(host: String, port: String, receiver: &Receiver<(u64, String)>) {
    let cloned_host = String::from(host.as_str());
    let cloned_port = String::from(port.as_str());
    let socket_address_option = http_utilities::create_socket_address(host, port);

    if socket_address_option.is_none() {
        eprintln!("An error occurred while creating socket address using host {cloned_host} and port {cloned_port}.");

        return;
    }

    let make_service = make_service_fn(|socket: &AddrStream| {
        let remote_address = socket.remote_addr();
        let receiver = receiver.clone();

        async move {
            Ok::<_, Infallible>(service_fn(move |request: Request<Body>| {
                let request_id = HTTP_REQUEST_COUNT.fetch_add(1_u64, Ordering::SeqCst);
                let receiver = receiver.clone();

                async move {
                    Ok::<_, Infallible>(
                        handle_request_async(request_id, remote_address, request, &receiver).await,
                    )
                }
            }))
        }
    });

    let socket_address = socket_address_option.unwrap();
    let server_builder = Server::bind(&socket_address);
    let server = server_builder.serve(make_service);

    println!("Server listening on http://{cloned_host}:{cloned_port}");

    if let Err(error) = server.await {
        eprintln!("An unexpected error occurred: {error}");
    }
}
