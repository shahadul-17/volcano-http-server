use hyper::{service::service_fn, Body, Request, Response, upgrade::Upgraded};
use serde_json::json;
use tokio_tungstenite::WebSocketStream;
use tungstenite::{Message, protocol::WebSocketConfig};
use std::{
    convert::Infallible,
    net::SocketAddr,
    borrow::BorrowMut,
};
use tokio::{net::TcpListener, sync::watch::Receiver};
use futures::{stream::StreamExt, SinkExt};

use crate::{
    http_server_configuration::HttpServerConfiguration,
    http_utilities::{self, SerializableHttpRequest, SerializableHttpResponse},
    web_socket_utilities,
    ipc_handler,
};

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

async fn handle_web_socket_stream_async(
    request_id: u64,
    remote_address: SocketAddr,
    _receiver: &Receiver<(u64, String)>,
    mut web_socket_stream: WebSocketStream<Upgraded>) {
    let mut message_count = 1_u64;
    println!("Web socket client connected from {}:{} with ID, '{}'.", remote_address.ip(), remote_address.port(), request_id);

    loop {
        let message_result_option = web_socket_stream.next().await;

        if message_result_option.is_none() { continue; }

        let message_result = message_result_option.unwrap();

        if message_result.is_err() {
            let error = message_result.unwrap_err();

            eprintln!("An error occurred while handling WebSocket Stream data: {error}");

            continue;
        }

        let message = message_result.unwrap();
        let message_id = message_count;

        // if message count is equal to the maximum value...
        if message_count == u64::MAX {
            // we shall reset the message count...
            message_count = 0_u64;
        }

        message_count = message_count + 1;

        match message {
            Message::Text(message) => {
                println!("[{}:{}:{}:{}] message: {}", message_id, request_id, remote_address.ip(), remote_address.port(), message);

                _ = web_socket_stream.send(Message::text(json!({
                    "requestId": request_id,
                    "messageId": message_id,
                    "remoteIpAddress": remote_address.ip(),
                    "remotePort": remote_address.port(),
                    "content": message,
                }).to_string())).await;
            },
            Message::Binary(message) => {
                println!("Received binary message: {:02X?}", message);
                // websocket.send(Message::binary(b"Thank you, come again.".to_vec())).await?;
            },
            Message::Ping(message) => {
                // No need to send a reply: tungstenite takes care of this for you.
                println!("Received ping message: {:02X?}", message);
            },
            Message::Pong(message) => {
                println!("Received pong message: {:02X?}", message);
            }
            Message::Close(close_frame_option) => {
                _ = web_socket_stream.close(close_frame_option.clone()).await;

                if close_frame_option.is_none() {
                    println!("Web socket client disconnected from {}:{} with ID, '{}'.", remote_address.ip(), remote_address.port(), request_id);
                } else {
                    let close_frame = close_frame_option.unwrap();

                    println!("Web socket client disconnected from {}:{} with ID, '{}'; status code, '{}' and reason, '{}'.", remote_address.ip(), remote_address.port(), request_id, close_frame.code, close_frame.reason);
                }

                break;
            },
            Message::Frame(_frame) => {
               unreachable!();
            },
        }
    }
}

async fn handle_request_async(
    request_id: u64,
    remote_address: SocketAddr,
    mut request: impl BorrowMut<Request<Body>>,
    receiver: &Receiver<(u64, String)>,
    configuration: &HttpServerConfiguration,
) -> Response<Body> {
    let receiver = receiver.clone();
    let configuration = configuration.clone();
    let borrowed_request: &mut Request<Body> = request.borrow_mut();
    let http_request =
        http_utilities::serialize_http_request_async(request_id, remote_address, borrowed_request).await;
    let is_upgrade_to_web_socket_request = web_socket_utilities::
        is_upgrade_request(&http_request.headers);

    // if web socket upgrade request is received...
    if is_upgrade_to_web_socket_request {
        // but web socket server is not enabled...
        if !configuration.is_web_socket_server_enabled {
            // we shall return erroneous response...
            return Response::builder()
                .status(400)
                .body(Body::from("Web socket server is not enabled."))
                .unwrap();
        }

        let upgrade_result = web_socket_utilities::
            upgrade(request, &http_request.headers);

        if upgrade_result.is_err() {
            let error = upgrade_result.unwrap_err();

            eprintln!("An error occurred while upgrading connection to WebSocket: {}", error);

            // we shall return erroneous response...
            return Response::builder()
                .status(400)
                .body(Body::from("An error occurred while upgrading connection to WebSocket."))
                .unwrap();
        }

        let (response, on_upgrade) = upgrade_result.unwrap();
        let web_socket_configuration: Option<WebSocketConfig> = None;

        // spawns a task...
        _ = tokio::spawn(async move {
            // retrieves the upgraded connection...
            let upgrade_option =
                web_socket_utilities::get_upgraded_connection(on_upgrade).await;

            if upgrade_option.is_none() { return; }

            let upgraded = upgrade_option.unwrap();
            let web_socket_stream = web_socket_utilities::to_web_socket_stream(upgraded, web_socket_configuration).await;

            // to handles web socket connection...
            handle_web_socket_stream_async(request_id, remote_address, &receiver, web_socket_stream).await;
        });

        return response;
    }

    let response = get_parent_process_response_async(&http_request, &receiver).await;

    return response;
}

pub async fn start_async(
    configuration: &HttpServerConfiguration,
    receiver: &Receiver<(u64, String)>,
) {
    let cloned_configuration = configuration.clone();
    let host = String::from(cloned_configuration.host.as_str());
    let port = cloned_configuration.port;
    let socket_address_result =
        http_utilities::create_socket_address(cloned_configuration.host, cloned_configuration.port);

    if socket_address_result.is_err() {
        let error = socket_address_result.unwrap_err();

        eprintln!(
            "An error occurred while creating socket address using host {} and port {}: {}",
            host, port, error
        );

        return;
    }

    let socket_address = socket_address_result.unwrap();
    let tcp_listener_result = TcpListener::bind(&socket_address).await;

    if tcp_listener_result.is_err() {
        let error = tcp_listener_result.unwrap_err();

        eprintln!("An error occurred while creating TCP listener: {}", error);

        return;
    }

    let tcp_listener = tcp_listener_result.unwrap();
    let http = http_utilities::create_http(configuration);
    let tls_acceptor_option = http_utilities::create_tls_acceptor(configuration);
    let mut request_count = 1_u64;

    println!();

    // checks if TLS is not enabled...
    if tls_acceptor_option.is_none() {
        println!("Server listening on http://{}:{}", host, port);
    } else {
        println!("Server listening on https://{}:{}", host, port);
    }

    loop {
        let accept_result = tcp_listener.accept().await;

        // in case of error...
        if accept_result.is_err() {
            let error = accept_result.unwrap_err();

            eprintln!("An error occurred while accepting connection: {}", error);

            // we shall skip this iteration...
            continue;
        }

        let receiver = receiver.clone();
        let http = http.clone();
        let tls_acceptor_option = tls_acceptor_option.clone();
        let cloned_configuration = configuration.clone();
        let request_id = request_count;

        // if request count is equal to the maximum value...
        if request_count == u64::MAX {
            // we shall reset the request count...
            request_count = 0_u64;
        }

        request_count = request_count + 1;

        tokio::spawn(async move {
            let http = http.clone();
            let (tcp_stream, remote_address) = accept_result.unwrap();
            let service_function = service_fn(move |request: Request<Body>| {
                let receiver = receiver.clone();
                let cloned_configuration = cloned_configuration.clone();

                async move {
                    Ok::<_, Infallible>(
                        handle_request_async(
                            request_id,
                            remote_address,
                            request,
                            &receiver,
                            &cloned_configuration,
                        )
                        .await,
                    )
                }
            });

            // if TLS acceptor is none...
            if tls_acceptor_option.is_none() {
                // we shall serve the connection without TLS...
                let connection = http
                    .serve_connection(tcp_stream, service_function)
                    .with_upgrades();
                let connection_result = connection.await;

                if connection_result.is_err() {
                    let error = connection_result.unwrap_err();

                    eprintln!("An unexpected connection error occurred: {}", error);
                }

                return;
            }

            // otherwise, we shall serve the connection with TLS...
            let tls_acceptor = tls_acceptor_option.unwrap();
            let accept_result = tls_acceptor.accept(tcp_stream).await;

            if accept_result.is_err() {
                let error = accept_result.unwrap_err();

                eprintln!(
                    "An error occurred while accepting TLS connection: {}",
                    error
                );

                return;
            }

            let tls_stream = accept_result.unwrap();
            let connection = http
                .serve_connection(tls_stream, service_function)
                .with_upgrades();
            let connection_result = connection.await;

            if connection_result.is_err() {
                let error = connection_result.unwrap_err();

                eprintln!("An unexpected connection error occurred: {}", error);
            }
        });
    }
}
