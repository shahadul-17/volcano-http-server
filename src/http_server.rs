#[path = "utilities/http_utilities.rs"]
mod http_utilities;

use hyper::{
    server::conn::Http,
    service::service_fn,
    Body, Request, Response,
};
use hyper_tungstenite::{is_upgrade_request, upgrade, tungstenite, HyperWebsocket};
use serde_json::json;
use tungstenite::Message;
use futures::{stream::StreamExt, SinkExt};

use std::{
    convert::Infallible,
    net::SocketAddr,
    sync::{atomic::{AtomicU64, Ordering}, Arc},
};
use tokio::{sync::watch, net::TcpListener};
use watch::Receiver;
use rustls::{Certificate, PrivateKey};
use tokio_rustls::TlsAcceptor;

use self::http_utilities::{SerializableHttpRequest, SerializableHttpResponse};
use crate::ipc_handler;

use std::vec::Vec;
use std::{fs::File, io::{self, BufReader}};

static HTTP_REQUEST_COUNT: AtomicU64 = AtomicU64::new(1_u64);

fn load_certificate_chain(file_path: String) -> io::Result<Vec<Certificate>> {
    let file = File::open(file_path)?;
    let mut buffered_reader = BufReader::new(file);
    let certificate_chain = rustls_pemfile::certs(&mut buffered_reader)?;

    return Ok(certificate_chain.into_iter().map(Certificate).collect());
}

fn load_private_key(file_path: String) -> io::Result<PrivateKey> {
    let file = File::open(file_path)?;
    let mut buffered_reader = BufReader::new(file);
    let private_keys = rustls_pemfile::rsa_private_keys(&mut buffered_reader)?;
    let private_key = PrivateKey(private_keys[0].clone());

    return Ok(private_key);
}

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

async fn handle_web_socket_async(request_id: u64,
    remote_address: SocketAddr,
    _request: Request<Body>,
    _receiver: &Receiver<(u64, String)>,
    websocket: HyperWebsocket) {
    let websocket_stream_result = websocket.await;

    if websocket_stream_result.is_err() {
        let error = websocket_stream_result.unwrap_err();

        eprintln!("An error occurred while handling Web Socket: {error}");

        return;
    }

    let mut websocket_stream = websocket_stream_result.unwrap();

    println!("Web socket client connected from {}:{} with ID, '{}'.", remote_address.ip(), remote_address.port(), request_id);

    loop {
        let message_result_option = websocket_stream.next().await;

        if message_result_option.is_none() { continue; }

        let message_result = message_result_option.unwrap();

        if message_result.is_err() {
            let error = message_result.unwrap_err();

            eprintln!("An error occurred while handling Web Socket Stream data: {error}");

            continue;
        }

        let message = message_result.unwrap();

        match message {
            Message::Text(message) => {
                println!("[{}:{}:{}] message: {}", request_id, remote_address.ip(), remote_address.port(), message);
                _ = websocket_stream.send(Message::text(json!({
                    "requestId": request_id,
                    "remoteIpAddress": remote_address.ip(),
                    "remotePort": remote_address.port(),
                    "content": message,
                }).to_string())).await;
            },
            Message::Binary(msg) => {
                println!("Received binary message: {:02X?}", msg);
                // websocket.send(Message::binary(b"Thank you, come again.".to_vec())).await?;
            },
            Message::Ping(msg) => {
                // No need to send a reply: tungstenite takes care of this for you.
                println!("Received ping message: {:02X?}", msg);
            },
            Message::Pong(msg) => {
                println!("Received pong message: {:02X?}", msg);
            }
            Message::Close(msg) => {
                _ = websocket_stream.close(msg).await;

                break;
                println!("[{}:{}:{}] Connection closed.", request_id, remote_address.ip(), remote_address.port());
                
                // No need to send a reply: tungstenite takes care of this for you.
                if let Some(msg) = &msg {
                    println!("Received close message with code {} and message: {}", msg.code, msg.reason);
                } else {
                    println!("Received close message");
                }
            },
            Message::Frame(_frame) => {
               unreachable!();
            },
        }
    }

    // while let Some(message_result) = websocket_stream.next().await {
    //     if message_result.is_err() {
    //         continue;
    //     }

    //     let message: Message = message_result.unwrap();

        
    // }
}

async fn handle_request_async(
    request_id: u64,
    remote_address: SocketAddr,
    mut request: Request<Body>,
    receiver: &Receiver<(u64, String)>,
) -> Response<Body> {
    let receiver = receiver.clone();

    // checks if the request is an upgrade request...
    if is_upgrade_request(&request) {
        let upgrade_result = upgrade(&mut request, None);

        if upgrade_result.is_err() {
            let error = upgrade_result.unwrap_err();

            eprintln!("An error occurred while upgrading the HTTP request: {error}");

            return Response::builder()
                .status(500)
                .body(Body::from("ERROR"))
                .unwrap();
        }

        let (response, hyper_web_socket) = upgrade_result.unwrap();

        // spawns a task...
        tokio::spawn(async move {
            // to handles web socket connection...
            handle_web_socket_async(request_id, remote_address,
                request, &receiver, hyper_web_socket).await;
        });

        // Return the response so the spawned future can continue.
        return response;
    }

    println!("Client connected from {}:{} with ID, '{}'.", remote_address.ip(), remote_address.port(), request_id);

    let http_request =
        http_utilities::serialize_http_request_async(request_id, remote_address, request).await;
    // let response = get_parent_process_response_async(&http_request, &receiver).await;
    let response = Response::builder()
        .status(200)
        .body(Body::from("Hello There!!!"))
        .unwrap();

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

    let socket_address = socket_address_option.unwrap();
    let tcp_listener_result = TcpListener::bind(&socket_address).await;

    if tcp_listener_result.is_err() {
        let error = tcp_listener_result.unwrap_err();

        eprintln!("An error occurred while creating new TCP listener: {error}");

        return;
    }

    let certificate_chain = load_certificate_chain(String::from("./certs/certificate.pem")).unwrap();
    let private_key = load_private_key(String::from("./certs/certificate.key")).unwrap();
    let tcp_listener = tcp_listener_result.unwrap();

    let mut config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certificate_chain, private_key).unwrap();
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec(), b"http/1.0".to_vec()];

    let tls_acceptor = TlsAcceptor::from(Arc::new(config));
    let http = Http::new();

    println!("Server listening on http://{cloned_host}:{cloned_port}");

    loop {
        let accept_result = tcp_listener.accept().await;

        if accept_result.is_err() {
            let error = accept_result.unwrap_err();

            eprintln!("An error occurred while accepting connection: {error}");

            return;
        }

        let receiver = receiver.clone();
        let http = http.clone();
        let tls_acceptor = tls_acceptor.clone();

        tokio::spawn(async move {
            let tls_acceptor = tls_acceptor.clone();
            let http = http.clone();
            let (tcp_stream, remote_address) = accept_result.unwrap();
            let accept_result = tls_acceptor.accept(tcp_stream).await;
            let tls_stream = accept_result.unwrap();

            let service_function = service_fn(move |request: Request<Body>| {
                let request_id = HTTP_REQUEST_COUNT.fetch_add(1_u64, Ordering::SeqCst);
                let receiver = receiver.clone();

                async move {
                    Ok::<_, Infallible>(
                        handle_request_async(request_id, remote_address, request, &receiver).await,
                    )
                }
            });

            let connection = http
                .serve_connection(tls_stream, service_function)
                .with_upgrades();

            if let Err(error) = connection.await {
                eprintln!("An unexpected error occurred: {error}");
            }
        });
    }
}
