#[path = "utilities/arguments_parser.rs"]
mod arguments_parser;
#[path = "http_server.rs"]
mod http_server;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: &str = "61798";

#[tokio::main]
async fn main() {
    let arguments = arguments_parser::populate_arguments_map();
    let host = arguments_parser::get_argument_or_default("host", DEFAULT_HOST, &arguments);
    let port = arguments_parser::get_argument_or_default("port", DEFAULT_PORT, &arguments);
    // let line_delimiter = arguments_parser::get_argument("lineDelimiter", &arguments);

    http_server::start_async(host, port).await;
}
