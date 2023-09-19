use crate::{arguments_parser::ArgumentsParser, system};

const DEFAULT_WORKER_THREAD_COUNT: &str = "16";
const DEFAULT_MAXIMUM_BLOCKING_THREAD_COUNT: &str = "1024";
const DEFAULT_BLOCKING_THREAD_KEEP_ALIVE_TIMEOUT_IN_MILLISECONDS: &str = "10000";
const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: &str = "61798";
const DEFAULT_POWERED_BY: &str = "Volcano";
const IS_WEB_SOCKET_SERVER_ENABLED_BY_DEFAULT: &str = "false";
const IS_TLS_ENABLED_BY_DEFAULT: &str = "false";
const DEFAULT_TLS_CERTIFICATE_PATH: &str = "tls_certificate.pem";
const DEFAULT_TLS_PRIVATE_KEY_PATH: &str = "tls_private_key.key";
const IS_HTTP2_ENABLED_BY_DEFAULT: &str = "false";

#[derive(Clone)]
pub struct Configuration {
    pub host: String,
    pub port: u16,
    pub powered_by: String,
    pub worker_thread_count: usize,
    pub maximum_blocking_thread_count: usize,
    pub blocking_thread_keep_alive_timeout_in_milliseconds: u64,
    pub is_web_socket_server_enabled: bool,
    pub is_tls_enabled: bool,
    pub tls_certificate_path: String,
    pub tls_private_key_path: String,
    pub is_http2_enabled: bool,
}

impl Configuration {
    pub fn from(arguments_parser: &ArgumentsParser) -> Self {
        let host = arguments_parser.get_as_string("host", DEFAULT_HOST);
        let port = arguments_parser.get_as_u16("port", DEFAULT_PORT);
        let powered_by = arguments_parser.get_as_string("poweredBy", DEFAULT_POWERED_BY);
        let worker_thread_count = arguments_parser.get_as_usize("workerThreadCount", DEFAULT_WORKER_THREAD_COUNT);
        let maximum_blocking_thread_count = arguments_parser.get_as_usize(
            "maximumBlockingThreadCount",
            DEFAULT_MAXIMUM_BLOCKING_THREAD_COUNT,
        );
        let blocking_thread_keep_alive_timeout_in_milliseconds = arguments_parser.get_as_u64(
            "blockingThreadKeepAliveTimeout",
            DEFAULT_BLOCKING_THREAD_KEEP_ALIVE_TIMEOUT_IN_MILLISECONDS,
        );
        let is_web_socket_server_enabled = arguments_parser.get_as_boolean(
            "enableWebSocketServer",
            IS_WEB_SOCKET_SERVER_ENABLED_BY_DEFAULT,
        );
        // uses TLS certificates...
        let is_tls_enabled = arguments_parser.get_as_boolean("enableTls", IS_TLS_ENABLED_BY_DEFAULT);
        let tls_certificate_path = arguments_parser.get_as_string("tlsCertificatePath", DEFAULT_TLS_CERTIFICATE_PATH);
        let tls_private_key_path = arguments_parser.get_as_string("tlsPrivateKeyPath", DEFAULT_TLS_PRIVATE_KEY_PATH);
        let is_http2_enabled = is_tls_enabled && arguments_parser.get_as_boolean("enableHttp2", IS_HTTP2_ENABLED_BY_DEFAULT);
        // prepares the configuration...
        let configuration = Configuration {
            host,
            port,
            powered_by,
            worker_thread_count,
            maximum_blocking_thread_count,
            blocking_thread_keep_alive_timeout_in_milliseconds,
            is_web_socket_server_enabled,
            is_tls_enabled,
            tls_certificate_path,
            tls_private_key_path,
            is_http2_enabled,
        };

        return configuration;
    }

    pub fn print(&self) {
        system::print_version_information();

        println!();
        println!("Host: {}", self.host);
        println!("Port: {}", self.port);
        println!("Powered by: {}", self.powered_by);
        println!("Worker thread count: {}", self.worker_thread_count);
        println!(
            "Maximum blocking thread count: {}",
            self.maximum_blocking_thread_count
        );
        println!(
            "Blocking thread keep alive timeout: {} ms",
            self.blocking_thread_keep_alive_timeout_in_milliseconds
        );
        println!(
            "WebSocket server enabled: {}",
            self.is_web_socket_server_enabled
        );
        println!("TLS enabled: {}", self.is_tls_enabled);

        if !self.is_tls_enabled {
            return;
        }

        println!("TLS certificate path: {}", self.tls_certificate_path);
        println!("TLS private key path: {}", self.tls_private_key_path);
        println!("HTTP/2 enabled: {}", self.is_http2_enabled);
    }
}
