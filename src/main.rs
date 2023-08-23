use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[path = "utilities/arguments_parser.rs"]
mod arguments_parser;
#[path = "http_server.rs"]
mod http_server;
#[path = "ipc_handler.rs"]
mod ipc_handler;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: &str = "61798";

#[tokio::main]
async fn main() {
    let arguments = arguments_parser::populate_arguments_map();
    let host = arguments_parser::get_argument_or_default("host", DEFAULT_HOST, &arguments);
    let port = arguments_parser::get_argument_or_default("port", DEFAULT_PORT, &arguments);
    // let line_delimiter = arguments_parser::get_argument("lineDelimiter", &arguments);
    let ipc_request_map: HashMap<String, String> = HashMap::new();
    let ipc_request_map_mutex: Mutex<HashMap<String, String>> = Mutex::new(ipc_request_map);
    let ipc_request_map_arc: Arc<Mutex<HashMap<String, String>>> = Arc::new(ipc_request_map_mutex);
    let join_handle = ipc_handler::start(&ipc_request_map_arc);

    http_server::start_async(host, port, &ipc_request_map_arc).await;

    _ = join_handle.join();
}
