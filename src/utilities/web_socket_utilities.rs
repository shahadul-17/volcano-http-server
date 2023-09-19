const CONNECTION_HEADER_NAME: &str = "connection";
const CONNECTION_HEADER_VALUE_UPGRADE: &str = "upgrade";
const UPGRADE_HEADER_NAME: &str = "upgrade";
const UPGRADE_HEADER_VALUE_WEBSOCKET: &str = "websocket";
const WEB_SOCKET_KEY_HEADER_NAME: &str = "sec-websocket-key";
const WEB_SOCKET_VERSION_HEADER_NAME: &str = "sec-websocket-version";
const WEB_SOCKET_ACCEPT_HEADER_NAME: &str = "sec-websocket-accept";
const SUPPORTED_WEB_SOCKET_VERSION: i32 = 13;

use std::{
    collections::HashMap,
    borrow::BorrowMut,
};
use hyper::{Request, Body, Response, upgrade::{self, OnUpgrade, Upgraded}};
use tokio_tungstenite::WebSocketStream;
use tungstenite::{
    protocol::{WebSocketConfig, Role},
    handshake,
    error::ProtocolError,
};

use crate::http_utilities;

pub fn is_upgrade_request(request_headers: &HashMap<String, Vec<String>>) -> bool {
    let connection_header_value = http_utilities::get_header_value(
        CONNECTION_HEADER_NAME, 0, request_headers).to_lowercase();

    if !connection_header_value.as_str().eq(CONNECTION_HEADER_VALUE_UPGRADE) {
        return false;
    }

    let upgrade_header_value = http_utilities::get_header_value(
        UPGRADE_HEADER_NAME, 0, request_headers).to_lowercase();

    if !upgrade_header_value.as_str().eq(UPGRADE_HEADER_VALUE_WEBSOCKET) {
        return false;
    }

    return true;
}

pub fn upgrade(mut request: impl BorrowMut<Request<Body>>,
    request_headers: &HashMap<String, Vec<String>>)
    -> Result<(Response<Body>, OnUpgrade), ProtocolError> {
    let web_socket_version_header_value = http_utilities::get_header_value(
        WEB_SOCKET_VERSION_HEADER_NAME, 0, request_headers);

    // if the web socket version header is missing...
    if web_socket_version_header_value.len() == 0 {
        // we shall return error...
        return Err(ProtocolError::MissingSecWebSocketVersionHeader);
    }

    let web_socket_version_parse_result = web_socket_version_header_value.parse::<i32>();

    // if the web socket version is invalid...
    if web_socket_version_parse_result.is_err() {
        let error = web_socket_version_parse_result.unwrap_err();

        eprintln!("An error occurred while parsing web socket version {} as integer: {}", web_socket_version_header_value, error);

        // we shall return error...
        return Err(ProtocolError::MissingSecWebSocketVersionHeader);
    }

    let web_socket_version = web_socket_version_parse_result.unwrap();

    // if the web socket version is not supported...
    if web_socket_version != SUPPORTED_WEB_SOCKET_VERSION {
        // we shall return error...
        return Err(ProtocolError::MissingSecWebSocketVersionHeader);
    }

    let web_socket_key_header_value = http_utilities::get_header_value(
        WEB_SOCKET_KEY_HEADER_NAME, 0, request_headers);

    // if the web socket key header is missing...
    if web_socket_key_header_value.len() == 0 {
        // we shall return error...
        return Err(ProtocolError::MissingSecWebSocketKey);
    }

    let web_socket_key_as_bytes = web_socket_key_header_value.as_bytes();
    let web_socket_key_response_header_value = handshake::
        derive_accept_key(web_socket_key_as_bytes);
    let response = Response::builder()
        .status(hyper::StatusCode::SWITCHING_PROTOCOLS)
        .header(CONNECTION_HEADER_NAME, CONNECTION_HEADER_VALUE_UPGRADE)
        .header(UPGRADE_HEADER_NAME, UPGRADE_HEADER_VALUE_WEBSOCKET)
        .header(WEB_SOCKET_ACCEPT_HEADER_NAME, &web_socket_key_response_header_value)
        .body(Body::empty())
        .unwrap();

    let request: &mut Request<Body> = request.borrow_mut();
    let on_upgrade = upgrade::on(request);

    return Ok((response, on_upgrade));
}

pub async fn get_upgraded_connection(on_upgrade: OnUpgrade) -> Option<Upgraded> {
    let upgrade_result = on_upgrade.await;

    if upgrade_result.is_err() {
        let error = upgrade_result.unwrap_err();

        eprintln!("An error occurred while upgrading connection to WebSocket: {}", error);

        // we shall return none...
        return None;
    }

    let upgraded = upgrade_result.unwrap();

    return Some(upgraded);
}

pub async fn to_web_socket_stream(upgraded: Upgraded, configuration: Option<WebSocketConfig>) -> WebSocketStream<Upgraded> {
    let web_socket_stream_future = WebSocketStream::from_raw_socket(
        upgraded,
        Role::Server,
        configuration,
    );
    let web_socket_stream = web_socket_stream_future.await;

    return web_socket_stream;
}
