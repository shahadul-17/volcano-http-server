#[path = "utilities/arguments_parser.rs"]
mod arguments_parser;
#[path = "configuration.rs"]
mod configuration;
#[path = "http_server.rs"]
mod http_server;
#[path = "ipc_handler.rs"]
mod ipc_handler;

use configuration::Configuration;
use std::time::Duration;
use tokio::sync::watch;

async fn main_async(configuration: &Configuration) {
    let configuration = configuration.clone();
    // let line_delimiter = arguments_parser::get_argument("lineDelimiter", &arguments);
    let (sender, receiver) = watch::channel((0u64, String::from("")));
    let join_handle = ipc_handler::start(sender);

    http_server::start_async(configuration.host, configuration.port, &receiver).await;

    _ = join_handle.join();
}

fn main() {
    let arguments = arguments_parser::populate_arguments_map();
    let configuration = configuration::get_configuration(&arguments);

    println!("[Configuration]");
    println!("Host: {}", configuration.host);
    println!("Port: {}", configuration.port);
    println!("Powered by: {}", configuration.powered_by);
    println!("Worker thread count: {}", configuration.worker_thread_count);
    println!(
        "Maximum blocking thread count: {}",
        configuration.maximum_blocking_thread_count
    );
    println!(
        "Blocking thread keep alive timeout: {} ms\n",
        configuration.blocking_thread_keep_alive_timeout_in_milliseconds
    );

    tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .worker_threads(configuration.worker_thread_count)
        .max_blocking_threads(configuration.maximum_blocking_thread_count)
        .thread_keep_alive(Duration::from_millis(
            configuration.blocking_thread_keep_alive_timeout_in_milliseconds,
        ))
        .build()
        .unwrap()
        .block_on(async {
            main_async(&configuration).await;
        });
}
