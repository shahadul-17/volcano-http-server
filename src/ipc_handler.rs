use std::collections::HashMap;
use std::io::{stdin, stdout, BufRead, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use futures::executor::block_on;

async fn listen_async(ipc_request_map_arc: &Arc<Mutex<HashMap<String, String>>>) {
    // let cloned_ipc_request_map_arc = ipc_request_map_arc.clone();
    let mut line_buffer = String::from("");
    let standard_input = stdin();
    let mut standard_input_lock = standard_input.lock();
    let cloned_ipc_request_map_arc = ipc_request_map_arc.clone();

    loop {
        let result = standard_input_lock.read_line(&mut line_buffer);

        if result.is_err() {
            let error = result.unwrap_err();

            eprintln!("An error occurred while reading line from the standard input: {error}");

            continue;
        }

        let line = line_buffer.trim().to_string();
        let line_as_str = String::from(line.as_str());

        line_buffer.clear();

        let mut ipc_request_map_mutex_guard = cloned_ipc_request_map_arc.lock().unwrap();
        ipc_request_map_mutex_guard.insert(line, line_as_str);
    }
}

pub fn write_line(text: String) -> bool {
    let mut text_to_write = String::from("");
    text_to_write.push_str(text.as_str());
    text_to_write.push('\n');
    let bytes = text_to_write.as_bytes();

    let mut standard_output = stdout().lock();
    let write_result = standard_output.write(bytes);

    if write_result.is_err() {
        return false;
    }

    _ = standard_output.flush();

    return true;
}

pub fn start(ipc_request_map_arc: &Arc<Mutex<HashMap<String, String>>>) -> JoinHandle<()> {
    let cloned_ipc_request_map_arc = ipc_request_map_arc.clone();
    let join_handle = thread::spawn(move || {
        block_on(listen_async(&cloned_ipc_request_map_arc));
    });

    return join_handle;
}
