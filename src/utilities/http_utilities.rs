use hyper::{
    body::HttpBody,
    http::HeaderValue,
    server::conn::Http,
    Body,
    HeaderMap,
    Request,
    Response,
};
use rustls::{Certificate, PrivateKey};
use serde::{Deserialize, Serialize};
use serde_json::{to_string, Value};
use std::{
    collections::HashMap,
    io,
    net::{AddrParseError, SocketAddr},
    str::FromStr,
    borrow::BorrowMut,
    sync::Arc,
};
use tokio_rustls::TlsAcceptor;
// use tokio::{io::{AsyncRead, AsyncWrite}, sync::watch::Receiver};
use urlencoding::decode;

use crate::{
    file_utilities,
    http_server_configuration::HttpServerConfiguration,
};

const BOUNDARY_MARKER: &str = "boundary=";
const BOUNDARY_MARKER_LENGTH: usize = BOUNDARY_MARKER.len();

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SerializableHttpRequest {
    pub request_id: u64,
    pub remote_ip_address: String,
    pub remote_port: i32,
    pub method: String,
    pub path: String,
    pub queries: HashMap<String, Vec<String>>,
    pub headers: HashMap<String, Vec<String>>,
    pub body_as_text: String,
    pub body: Value,
    pub url_encoded_from_data: HashMap<String, Vec<String>>,
}

impl SerializableHttpRequest {
    pub fn to_json(&self) -> String {
        let result = to_string(self);

        if result.is_err() {
            let error = result.unwrap_err();

            eprintln!(
                "An error occurred while serializing the HTTP request: {}",
                error
            );

            return String::from("");
        }

        return result.unwrap();
    }

    pub fn to_string(&self) -> String {
        return self.to_json();
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SerializableHttpResponse {
    pub request_id: u64,
    pub status_code: u16,
    pub headers: HashMap<String, Vec<String>>,
    pub body: Value,
}

impl SerializableHttpResponse {
    pub fn from(json: String) -> Option<Self> {
        let deserialization_result: Result<Self, serde_json::Error> = serde_json::from_str(&json);

        if deserialization_result.is_err() {
            let error = deserialization_result.unwrap_err();

            eprintln!(
                "An error occurred while deserializing into HTTP response: {}",
                error
            );

            return None;
        }

        return Some(deserialization_result.unwrap());
    }

    pub fn to_response(&self) -> Response<Body> {
        let headers = self.headers.iter();
        let mut response = Response::builder().status(self.status_code);

        for (header_name, header_values) in headers {
            for header_value in header_values {
                response = response.header(header_name.to_owned(), header_value.to_owned());
            }
        }

        let body_as_json = self.body.to_string();
        let body = Body::from(body_as_json);

        return response.body(body).unwrap();
    }
}

pub fn create_socket_address(host: String, port: u16) -> Result<SocketAddr, AddrParseError> {
    let mut socket_address_as_string: String = host;
    socket_address_as_string.push_str(":");
    socket_address_as_string.push_str(port.to_string().as_str());
    let socket_address_result = SocketAddr::from_str(&socket_address_as_string);

    return socket_address_result;
}

pub async fn to_serializable_header_map(
    header_map: &HeaderMap<HeaderValue>,
) -> HashMap<String, Vec<String>> {
    let mut headers: HashMap<String, Vec<String>> = HashMap::with_capacity(header_map.keys_len());
    let header_keys = header_map.keys();

    for header_key in header_keys {
        let header_name = header_key.as_str().to_lowercase();
        let header_values_iterator = header_map.get_all(&header_name);
        let mut header_values: Vec<String> = Vec::new();

        for header_value in header_values_iterator {
            let header_value = header_value.to_str().unwrap().to_owned();

            header_values.push(header_value);
        }

        headers.insert(header_name, header_values);
    }

    return headers;
}

pub async fn parse_url_encoded_string_async(
    url_encoded_string: &str,
) -> HashMap<String, Vec<String>> {
    let mut url_encoded_data_map: HashMap<String, Vec<String>> = HashMap::new();

    if url_encoded_string.len() == 0 {
        return url_encoded_data_map;
    }

    let splitted_url_encoded_string = url_encoded_string.split("&");

    for url_encoded_string in splitted_url_encoded_string {
        let splitted_data: Vec<&str> = url_encoded_string.split("=").collect();
        let key = decode(splitted_data[0]).unwrap().into_owned(); // splitted_query[0].to_owned();
        let value = decode(splitted_data[1]).unwrap().into_owned(); // splitted_query[1].to_owned();
        let values_option = url_encoded_data_map.get_mut(&key);

        if values_option.is_none() {
            let mut values: Vec<String> = Vec::new();
            values.push(value);
            url_encoded_data_map.insert(key, values);
        } else {
            let values = values_option.unwrap();
            values.push(value);
        }
    }

    return url_encoded_data_map;
}

pub fn get_header_value<'a>(
    header_name: &str,
    index_of_value: usize,
    headers: &'a HashMap<String, Vec<String>>,
) -> &'a str {
    let option = headers.get(header_name);

    if option.is_none() {
        return "";
    }

    let values = option.unwrap();
    let value_option = values.get(index_of_value);

    if value_option.is_none() {
        return "";
    }

    return value_option.unwrap();
}

pub async fn parse_body_as_text_async(_content_type: &str, body: &mut Body) -> String {
    let mut body_as_text = String::from("");

    while let Some(chunk) = HttpBody::data(body).await {
        if chunk.is_err() {
            let error = chunk.unwrap_err();

            eprintln!("An error occurred while reading body as text: {}", error);

            return String::from("");
        }

        let bytes = chunk.unwrap();

        // println!("Reading {} bytes", bytes.len());

        let bytes_as_vector = bytes.to_vec();
        let bytes_to_string_conversion_result = String::from_utf8(bytes_as_vector);

        if bytes_to_string_conversion_result.is_err() {
            let error = bytes_to_string_conversion_result.unwrap_err();

            eprintln!(
                "An error occurred while converting bytes to UTF-8 string: {}",
                error
            );

            return String::from("");
        }

        let chunk_as_string = bytes_to_string_conversion_result.unwrap();

        body_as_text.push_str(chunk_as_string.as_str());
    }

    return body_as_text;
}

pub async fn parse_body_as_multipart_form_data_async(content_type: &str, body: &mut Body) {
    let index_of_boundary_marker_option = content_type.find(BOUNDARY_MARKER);

    if index_of_boundary_marker_option.is_none() {
        return;
    }

    let index_of_boundary_marker = index_of_boundary_marker_option.unwrap();
    let boundary = &content_type[index_of_boundary_marker + BOUNDARY_MARKER_LENGTH..];
    let _boundary_as_bytes = boundary.as_bytes();

    println!("{} is the boundary.", boundary);

    let mut total_bytes_read = 0;

    while let Some(chunk) = HttpBody::data(body).await {
        if chunk.is_err() {
            let error = chunk.unwrap_err();

            eprintln!("An error occurred while reading body as text: {}", error);

            return;
        }

        let bytes = chunk.unwrap();
        total_bytes_read = total_bytes_read + bytes.len();
    }

    println!("Total {total_bytes_read} bytes read.");
}

// pub async fn parse_body_async(content_type: String, body: &mut Body) {
//     if content_type.is_empty() {
//         return;
//     }
// }

pub async fn serialize_http_request_async(
    request_id: u64,
    remote_address: SocketAddr,
    mut request: impl BorrowMut<Request<Body>>,
) -> SerializableHttpRequest {
    let request: &mut Request<Body> = request.borrow_mut();
    let mut remote_ip_address = remote_address.ip().to_string();
    let remote_port = i32::from(remote_address.port());
    let method = request.method().as_str().to_owned();
    let path = request.uri().path().to_owned();
    let query_string = if request.uri().query().is_none() {
        ""
    } else {
        request.uri().query().unwrap()
    };
    let queries = parse_url_encoded_string_async(query_string).await;
    let headers = to_serializable_header_map(request.headers()).await;
    let forwarded_for = get_header_value("x-forwarded-for", 0, &headers);

    if forwarded_for.len() > 0 {
        remote_ip_address = forwarded_for.to_owned();
    }

    let mut body_as_text = String::from("");
    let mut body: Value = Value::Null;
    let mut url_encoded_form_data: HashMap<String, Vec<String>> = HashMap::new();
    let content_type = get_header_value("content-type", 0, &headers);
    let is_json_content = content_type.contains("json");
    let is_text_content = content_type.contains("text");
    let is_url_encoded_form_data = "application/x-www-form-urlencoded".eq(content_type);
    let is_multipart_form_data = content_type.starts_with("multipart/form-data");
    let shall_parse_body_as_text = is_json_content || is_text_content || is_url_encoded_form_data;

    // if content type is any of the text types or JSON types,
    // we shall parse the body as text...
    if shall_parse_body_as_text {
        body_as_text = parse_body_as_text_async(content_type, request.body_mut()).await;

        if is_json_content {
            let result = Value::from_str(&body_as_text);

            if result.is_err() {
                let error = result.unwrap_err();

                eprintln!(
                    "An error occurred while serializing body as JSON: {}",
                    error
                );
            } else {
                body = result.unwrap();
            }
        } else if is_url_encoded_form_data {
            url_encoded_form_data = parse_url_encoded_string_async(body_as_text.as_str()).await;
        }
    }
    // else if the content is multipart form data...
    else if is_multipart_form_data {
        // body = parse_body_as_text_async(content_type, request.body_mut()).await;

        // println!("{body}");

        parse_body_as_multipart_form_data_async(content_type, request.body_mut()).await;
    }

    // println!("{content_type}");

    return SerializableHttpRequest {
        request_id,
        remote_ip_address,
        remote_port,
        method,
        path,
        queries,
        headers,
        body_as_text,
        body,
        url_encoded_from_data: url_encoded_form_data,
    };
}

fn load_tls_certificate_chain(file_path: String) -> io::Result<Vec<Certificate>> {
    let mut buffered_reader = file_utilities::create_buffered_file_reader(file_path)?;
    let certificates = rustls_pemfile::certs(&mut buffered_reader)?;

    if certificates.len() == 0 {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No certificate found.",
        ));
    }

    let certificate_chain = certificates.into_iter().map(Certificate).collect();

    return Ok(certificate_chain);
}

fn load_tls_private_key(file_path: String) -> io::Result<PrivateKey> {
    let mut buffered_reader = file_utilities::create_buffered_file_reader(file_path)?;
    let private_keys = rustls_pemfile::rsa_private_keys(&mut buffered_reader)?;

    if private_keys.len() == 0 {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No private key found.",
        ));
    }

    let private_key = PrivateKey(private_keys[0].clone());

    return Ok(private_key);
}

pub fn create_http(configuration: &HttpServerConfiguration) -> Http {
    // let configuration = configuration.clone();
    let mut http = Http::new();

    // if HTTP/2 is not enabled...
    if !configuration.is_http2_enabled {
        // we shall set HTTP/1 only to true...
        http.http1_only(true);
    }

    return http;
}

pub fn create_tls_acceptor(configuration: &HttpServerConfiguration) -> Option<TlsAcceptor> {
    let configuration = configuration.clone();

    // if TLS is not enabled...
    if !configuration.is_tls_enabled {
        // we shall return none...
        return None;
    }

    let tls_certificate_chain_result =
        load_tls_certificate_chain(configuration.tls_certificate_path);

    if tls_certificate_chain_result.is_err() {
        let error = tls_certificate_chain_result.unwrap_err();

        eprintln!(
            "An error occurred while loading TLS certificate chain: {}",
            error
        );

        return None;
    }

    let tls_private_key_result = load_tls_private_key(configuration.tls_private_key_path);

    if tls_private_key_result.is_err() {
        let error = tls_private_key_result.unwrap_err();

        eprintln!("An error occurred while loading TLS private key: {}", error);

        return None;
    }

    let tls_certificate_chain = tls_certificate_chain_result.unwrap();
    let tls_private_key = tls_private_key_result.unwrap();
    let tls_server_configuration_result = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(tls_certificate_chain, tls_private_key);

    if tls_server_configuration_result.is_err() {
        let error = tls_server_configuration_result.unwrap_err();

        eprintln!(
            "An error occurred while creating TLS server configuration: {}",
            error
        );

        return None;
    }

    let mut alpn_protocols = vec![
        b"h2".to_vec(),             // index 0...
        b"http/1.1".to_vec(),       // index 1...
        b"http/1.0".to_vec(),       // index 2...
    ];

    // if HTTP/2 is not enabled...
    if !configuration.is_http2_enabled {
        // we shall remove HTTP/2 from the alpn protocols...
        alpn_protocols.remove(0);
    }

    let mut tls_server_configuration = tls_server_configuration_result.unwrap();
    tls_server_configuration.alpn_protocols = alpn_protocols;
    let tls_acceptor = TlsAcceptor::from(Arc::new(tls_server_configuration));

    return Some(tls_acceptor);
}
