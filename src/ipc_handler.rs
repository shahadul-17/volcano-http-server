use std::collections::HashMap;
use std::io::{stdin, stdout, BufRead, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

const KEY_SEPARATOR: &str = "#";
const KEY_SEPARATOR_LENGTH: usize = KEY_SEPARATOR.len();

pub struct IpcOptions {
    pub line_map: HashMap<u64, String>,
    pub line_read_callback_map: Option<HashMap<u64, Box<dyn Fn(String) + Send + Sync>>>,
}

impl IpcOptions {
    pub fn new() -> Self {
        return IpcOptions {
            line_map: HashMap::new(),
            line_read_callback_map: Some(HashMap::new()),
        };
    }

    pub fn get_line(&mut self, key: &u64) -> String {
        let value_option = self.line_map.get(key);

        if value_option.is_none() {
            return String::from("");
        }

        let value = value_option.unwrap().to_owned();

        // removes the line from the map once the line is retrieved...
        self.line_map.remove(key);

        return value;
    }

    fn add_line(&mut self, key: u64, value: String) {
        self.line_map.insert(key, value);
    }

    #[allow(dead_code)]
    pub fn add_line_read_callback(
        &mut self,
        key: u64,
        callback: Box<dyn Fn(String) + Send + Sync>,
    ) {
        let line_read_callback_map = self.line_read_callback_map.as_mut().unwrap();

        line_read_callback_map.insert(key, callback);
    }

    fn execute_line_read_callback(&mut self, key: &u64, line: String) {
        let line_read_callback_map = self.line_read_callback_map.as_mut().unwrap();
        let line_read_callback_option = line_read_callback_map.get(key);

        if line_read_callback_option.is_none() {
            return;
        }

        let cloned_line = String::from(line.as_str());
        let line_read_callback = line_read_callback_option.unwrap();
        line_read_callback(cloned_line);

        // finally we need to remove the callback from the map...
        _ = line_read_callback_map.remove(key);
    }
}

fn listen(ipc_options_arc: &Arc<Mutex<IpcOptions>>) {
    let mut line_buffer = String::from("");
    let standard_input = stdin();
    let mut standard_input_lock = standard_input.lock();
    let cloned_ipc_options_arc = ipc_options_arc.clone();

    loop {
        // before we begin, we must clear the buffer...
        line_buffer.clear();

        let result = standard_input_lock.read_line(&mut line_buffer);

        if result.is_err() {
            let error = result.unwrap_err();

            eprintln!("An error occurred while reading line from the standard input: {error}");

            continue;
        }

        let line = line_buffer.trim().to_string();
        let index_of_key_separator_option = line.find(KEY_SEPARATOR);

        // if key separator is not found...
        if index_of_key_separator_option.is_none() {
            // we shall print an error message and continue to the next iteration...
            eprintln!("Invalid line read from the standard input: {line}");

            continue;
        }

        let index_of_key_separator = index_of_key_separator_option.unwrap();
        // we shall extract key from the received line...
        let key_extraction_result = String::from(&line[..index_of_key_separator]).parse::<u64>();

        if key_extraction_result.is_err() {
            let error = key_extraction_result.unwrap_err();

            // we shall print an error message and continue to the next iteration...
            eprintln!("An error occurred during key extraction: {error}");

            continue;
        }

        let key = key_extraction_result.unwrap();
        let line_without_key = String::from(&line[index_of_key_separator + KEY_SEPARATOR_LENGTH..]);
        let mut ipc_request_map_mutex_guard = cloned_ipc_options_arc.lock().unwrap();
        ipc_request_map_mutex_guard.add_line(key, String::from(line_without_key.as_str()));
        ipc_request_map_mutex_guard
            .execute_line_read_callback(&key, String::from(line_without_key.as_str()));
    }
}

pub async fn read_line_async(ipc_options_arc: Arc<Mutex<IpcOptions>>, key: u64) -> String {
    // let cloned_ipc_options_arc = ipc_options_arc.clone();
    let mut iteration_count = 0;

    loop {
        let mut ipc_request_map_mutex_guard = ipc_options_arc.lock().unwrap();
        let line = ipc_request_map_mutex_guard.get_line(&key);

        if !line.is_empty() {
            return line;
        }

        iteration_count = iteration_count + 1;

        if iteration_count > 150 {
            iteration_count = 0;

            thread::sleep(Duration::from_millis(5));
        }
    }
}

pub fn write_line(text: &String) -> bool {
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

pub fn start(ipc_options_arc: &Arc<Mutex<IpcOptions>>) -> JoinHandle<()> {
    let cloned_ipc_options_arc = ipc_options_arc.clone();
    let join_handle = thread::spawn(move || {
        listen(&cloned_ipc_options_arc);
        // block_on(listen_async(&cloned_ipc_options_arc));
    });

    return join_handle;
}
