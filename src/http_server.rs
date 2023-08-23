#[path = "utilities/http_utilities.rs"]
mod http_utilities;

use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::{
    collections::HashMap,
    convert::Infallible,
    net::SocketAddr,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::ipc_handler;
use self::http_utilities::SerializableHttpPayload;

static HTTP_REQUEST_COUNT: AtomicU64 = AtomicU64::new(1_u64);

async fn get_parent_process_response_async(
    http_request: &SerializableHttpPayload,
    ipc_request_map_arc: &Arc<Mutex<HashMap<String, String>>>,
) -> Response<Body> {
    let request_id = http_request
        .headers
        .get("x-request-id")
        .unwrap()
        .get(0)
        .unwrap()
        .to_owned();
    let http_request_as_json = http_request.to_string();
    let cloned_http_request_as_json = String::from(http_request_as_json.as_str());

    ipc_handler::write_line(http_request_as_json);

    let cloned_ipc_request_map_arc = ipc_request_map_arc.clone();
    let payload_id_as_string = request_id; // http_request.payload_id.to_string();
    let response_data: String;
    let mut i = 0;

    loop {
        let ipc_request_map_mutex_guard = cloned_ipc_request_map_arc.lock().unwrap();
        let response_data_option = ipc_request_map_mutex_guard.get(&payload_id_as_string);

        if !response_data_option.is_none() {
            response_data = response_data_option.unwrap().to_owned();

            break;
        }

        i += 1;

        if i < 1_00_000 {
            thread::sleep(Duration::from_millis(5));
        }
    }

    let response = Response::builder()
        .status(200)
        .header("X-Response-Data", response_data)
        .header("X-Powered-By", "Node.js")
        .header("Content-Type", "application/json")
        // .header("Set-Cookie", "hello=world")
        // .header("Set-Cookie", "how=are")
        // .body(Body::from("Hello World from RUST HTTP server..!!"));
        .body(Body::from(cloned_http_request_as_json))
        .unwrap();

    return response;
}

async fn handle_request_async(
    request_id: u64,
    remote_address: SocketAddr,
    request: Request<Body>,
    ipc_request_map_arc: &Arc<Mutex<HashMap<String, String>>>,
) -> Response<Body> {
    let cloned_ipc_request_map_arc = ipc_request_map_arc.clone();
    let http_request =
        http_utilities::serialize_http_request_async(request_id, remote_address, request).await;
    let response = tokio::spawn(async move {
        get_parent_process_response_async(&http_request, &cloned_ipc_request_map_arc).await
    })
    .await
    .unwrap();

    return response;
}

pub async fn start_async(
    host: String,
    port: String,
    ipc_request_map_arc: &Arc<Mutex<HashMap<String, String>>>,
) {
    let socket_address_option = http_utilities::create_socket_address(host, port);

    if socket_address_option.is_none() {
        eprintln!("An error occurred while creating socket address.");

        return;
    }

    let make_service = make_service_fn(|socket: &AddrStream| {
        let remote_address = socket.remote_addr();
        let cloned_ipc_request_map_arc = ipc_request_map_arc.clone();

        async move {
            Ok::<_, Infallible>(service_fn(move |request: Request<Body>| {
                let request_id = HTTP_REQUEST_COUNT.fetch_add(1_u64, Ordering::SeqCst);
                let cloned_ipc_request_map_arc = cloned_ipc_request_map_arc.clone();

                async move {
                    Ok::<_, Infallible>(
                        handle_request_async(
                            request_id,
                            remote_address,
                            request,
                            &cloned_ipc_request_map_arc,
                        )
                        .await,
                    )
                }
            }))
        }
    });

    let socket_address = socket_address_option.unwrap();
    let server_builder = Server::bind(&socket_address);
    let server = server_builder.serve(make_service);

    if let Err(error) = server.await {
        eprintln!("An unexpected error occurred: {error}");
    }
}
