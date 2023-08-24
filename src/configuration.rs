use std::collections::HashMap;

use crate::arguments_parser;

const DEFAULT_WORKER_THREAD_COUNT: &str = "16";
const DEFAULT_MAXIMUM_BLOCKING_THREAD_COUNT: &str = "1024";
const DEFAULT_BLOCKING_THREAD_KEEP_ALIVE_TIMEOUT_IN_MILLISECONDS: &str = "10";
const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: &str = "61798";

#[derive(Clone)]
pub struct Configuration {
    pub host: String,
    pub port: String,
    pub worker_thread_count: usize,
    pub maximum_blocking_thread_count: usize,
    pub blocking_thread_keep_alive_timeout_in_milliseconds: u64,
}

pub fn get_configuration(arguments: &HashMap<String, String>) -> Configuration {
    let host = arguments_parser::get_argument_or_default("host", DEFAULT_HOST, &arguments);
    let port = arguments_parser::get_argument_or_default("port", DEFAULT_PORT, &arguments);
    let worker_thread_count = arguments_parser::get_argument_or_default(
        "workerThreadCount",
        DEFAULT_WORKER_THREAD_COUNT,
        &arguments,
    )
    .parse::<usize>()
    .unwrap();
    let maximum_blocking_thread_count = arguments_parser::get_argument_or_default(
        "maximumBlockingThreadCount",
        DEFAULT_MAXIMUM_BLOCKING_THREAD_COUNT,
        &arguments,
    )
    .parse::<usize>()
    .unwrap();
    let blocking_thread_keep_alive_timeout_in_milliseconds =
        arguments_parser::get_argument_or_default(
            "blockingThreadKeepAliveTimeoutInMilliseconds",
            DEFAULT_BLOCKING_THREAD_KEEP_ALIVE_TIMEOUT_IN_MILLISECONDS,
            &arguments,
        )
        .parse::<u64>()
        .unwrap();

    let configuration = Configuration {
        host,
        port,
        worker_thread_count,
        maximum_blocking_thread_count,
        blocking_thread_keep_alive_timeout_in_milliseconds,
    };

    return configuration;
}
