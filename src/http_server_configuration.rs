use crate::configuration::Configuration;

#[derive(Clone)]
pub struct HttpServerConfiguration {
    pub host: String,
    pub port: u16,
    pub powered_by: String,
    pub is_web_socket_server_enabled: bool,
    pub is_tls_enabled: bool,
    pub tls_certificate_path: String,
    pub tls_private_key_path: String,
    pub is_http2_enabled: bool,
}

impl HttpServerConfiguration {
    pub fn from(configuration: Configuration) -> Self {
        let http_server_configuration = HttpServerConfiguration {
            host: configuration.host,
            port: configuration.port,
            powered_by: configuration.powered_by,
            is_web_socket_server_enabled: configuration.is_web_socket_server_enabled,
            is_tls_enabled: configuration.is_tls_enabled,
            tls_certificate_path: configuration.tls_certificate_path,
            tls_private_key_path: configuration.tls_private_key_path,
            is_http2_enabled: configuration.is_http2_enabled,
        };

        return http_server_configuration;
    }
}
