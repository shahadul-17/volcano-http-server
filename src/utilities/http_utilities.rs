use hyper::{body::HttpBody, http::HeaderValue, Body, HeaderMap, Request};
use serde::{Deserialize, Serialize};
use serde_json::{to_string, Value};
use std::{collections::HashMap, net::SocketAddr, str::FromStr};
use urlencoding::decode;

const BOUNDARY_MARKER: &str = "boundary=";
const BOUNDARY_MARKER_LENGTH: usize = BOUNDARY_MARKER.len();

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializableHttpPayload {
    pub payload_id: u64,
    pub remote_ip_address: String,
    pub remote_port: i32,
    pub method: String,
    pub path: String,
    pub queries: HashMap<String, Vec<String>>,
    pub headers: HashMap<String, Vec<String>>,
    pub body: String,
    pub body_as_object: Value,
    pub url_encoded_from_data: HashMap<String, Vec<String>>,
}

impl SerializableHttpPayload {
    pub fn to_json(&self) -> String {
        let result = to_string(self);

        if result.is_err() {
            let error = result.unwrap_err();

            eprintln!("An error occurred while serializing the http payload: {error}");

            return String::from("");
        }

        return result.unwrap();
    }

    pub fn to_string(&self) -> String {
        return self.to_json();
    }
}

pub fn create_socket_address(host: String, port: String) -> Option<SocketAddr> {
    let mut socket_address_as_string: String = host;
    socket_address_as_string.push_str(":");
    socket_address_as_string.push_str(&port);

    let socket_address_result = SocketAddr::from_str(&socket_address_as_string);

    if socket_address_result.is_err() {
        let error = socket_address_result.unwrap_err();

        eprintln!("An error occurred while parsing the provided bind address: {error}");

        return None;
    }

    return Some(socket_address_result.unwrap());
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

            eprintln!("An error occurred while reading body as text: {error}");

            return String::from("");
        }

        let bytes = chunk.unwrap();

        // println!("Reading {} bytes", bytes.len());

        let bytes_as_vector = bytes.to_vec();
        let bytes_to_string_conversion_result = String::from_utf8(bytes_as_vector);

        if bytes_to_string_conversion_result.is_err() {
            let error = bytes_to_string_conversion_result.unwrap_err();

            eprintln!("An error occurred while converting bytes to UTF-8 string: {error}");

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

            eprintln!("An error occurred while reading body as text: {error}");

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
    mut request: Request<Body>,
) -> SerializableHttpPayload {
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

    let mut body = String::from("");
    let mut body_as_object: Value = Value::Null;
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
        body = parse_body_as_text_async(content_type, request.body_mut()).await;

        if is_json_content {
            let result = Value::from_str(&body);

            if result.is_err() {
                let error = result.unwrap_err();

                eprintln!("An error occurred while serializing body as JSON: {error}");
            } else {
                body_as_object = result.unwrap();
            }
        } else if is_url_encoded_form_data {
            url_encoded_form_data = parse_url_encoded_string_async(body.as_str()).await;
        }
    }
    // else if the content is multipart form data...
    else if is_multipart_form_data {
        // body = parse_body_as_text_async(content_type, request.body_mut()).await;

        // println!("{body}");

        parse_body_as_multipart_form_data_async(content_type, request.body_mut()).await;
    }

    // println!("{content_type}");

    return SerializableHttpPayload {
        payload_id: request_id,
        remote_ip_address,
        remote_port,
        method,
        path,
        queries,
        headers,
        body,
        body_as_object,
        url_encoded_from_data: url_encoded_form_data,
    };
}
