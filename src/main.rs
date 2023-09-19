#[path = "utilities/arguments_parser.rs"]
mod arguments_parser;
#[path = "utilities/file_utilities.rs"]
mod file_utilities;
#[path = "utilities/http_utilities.rs"]
mod http_utilities;
#[path = "utilities/web_socket_utilities.rs"]
mod web_socket_utilities;
#[path = "configuration.rs"]
mod configuration;
#[path = "system.rs"]
mod system;
#[path = "http_server_configuration.rs"]
mod http_server_configuration;
#[path = "http_server.rs"]
mod http_server;
#[path = "ipc_handler.rs"]
mod ipc_handler;

use std::time::Duration;
use tokio::sync::watch;

use crate::{
    arguments_parser::ArgumentsParser,
    configuration::Configuration,
    http_server_configuration::HttpServerConfiguration,
};

async fn main_async(configuration: &Configuration) {
    let configuration = configuration.clone();
    // let line_delimiter = arguments_parser::get_argument("lineDelimiter", &arguments);
    let (sender, receiver) = watch::channel((0u64, String::from("")));
    let join_handle = ipc_handler::start(sender);
    let http_server_configuration = HttpServerConfiguration::from(configuration);

    http_server::start_async(&http_server_configuration, &receiver).await;

    _ = join_handle.join();
}

fn main() {
    let name_only_arguments = system::get_name_only_arguments();
    let arguments_parser = ArgumentsParser::new(&name_only_arguments);

    if arguments_parser.get_as_boolean("version", "false") {
        system::print_version_information();

        return;
    }

    if arguments_parser.get_as_boolean("help", "false") {
        system::print_help();

        return;
    }

    let configuration = Configuration::from(&arguments_parser);
    configuration.print();

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
