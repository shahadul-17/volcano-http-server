const VERSION: &str = "0.0.1";

pub fn get_name_only_arguments() -> Vec<String> {
    return vec![
        // these arguments do not have values...
        "help".to_string(),
        "version".to_string(),
        "enableWebSocketServer".to_string(),
        "enableTls".to_string(),
        "enableHttp2".to_string(),
    ];
}

pub fn print_help() {
    print_version_information();

    println!();
    println!("Usage: volcano-http-server <argument-name> <argument-value>\n");
    println!("Arguments");
    println!("--help                             Prints help.                                          Example: volcano-http-server --help");
    println!("--version                          Prints version information.                           Example: volcano-http-server --version");
    println!("--host                             Sets the address on which the server shall bind to.   Example: volcano-http-server --host 127.0.0.1");
    println!("--port                             Sets the port on which the server shall listen.       Example: volcano-http-server --port 61798");
    println!("--poweredBy                        Sets default X-Powered-By header.                     Example: volcano-http-server --poweredBy Volcano");
    println!("--workerThreadCount                Sets the number of worker threads to use.             Example: volcano-http-server --workerThreadCount 16");
    println!("--enableWebSocketServer            Enables the WebSocket server.                         Example: volcano-http-server --enableWebSocketServer");
    println!("--enableTls                        Enables HTTPS.                                        Example: volcano-http-server --enableTls");
    println!("--tlsCertificatePath               Sets the TLS certificate path.                        Example: volcano-http-server --tlsCertificatePath tls_certificate.pem");
    println!("--tlsPrivateKeyPath                Sets the TLS private key path.                        Example: volcano-http-server --tlsPrivateKeyPath tls_private_key.key");
    println!("--enableHttp2                      Enables HTTP/2 (TLS must be enabled).                 Example: volcano-http-server --enableTls --enableHttp2");
    println!("--maximumBlockingThreadCount       Sets the maximum number of blocking threads to use.   Example: volcano-http-server --maximumBlockingThreadCount 1024");
    println!("--blockingThreadKeepAliveTimeout   Sets keep alive timeout in milliseconds for the       Example: volcano-http-server --blockingThreadKeepAliveTimeout 10000");
    println!("                                   blocking threads.");
    println!();
}

pub fn print_version_information() {
    println!("Volcano HTTP Server v{}", VERSION);
}
