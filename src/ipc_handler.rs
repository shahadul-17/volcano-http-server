use std::io::{stdin, stdout, BufRead, Write};
use std::thread;
use std::thread::JoinHandle;
use tokio::sync::watch;
use watch::Receiver;
use watch::Sender;

const KEY_SEPARATOR: &str = "#";
const KEY_SEPARATOR_LENGTH: usize = KEY_SEPARATOR.len();

fn listen(sender: &Sender<(u64, String)>) {
    let sender = sender.clone();
    let mut line_buffer = String::from("");
    let standard_input = stdin();
    let mut standard_input_lock = standard_input.lock();

    loop {
        // before we begin, we must clear the buffer...
        line_buffer.clear();

        let result = standard_input_lock.read_line(&mut line_buffer);

        if result.is_err() {
            let error = result.unwrap_err();

            eprintln!("An error occurred while reading line from the standard input: {}", error);

            continue;
        }

        let line = line_buffer.trim().to_string();
        let index_of_key_separator_option = line.find(KEY_SEPARATOR);

        // if key separator is not found...
        if index_of_key_separator_option.is_none() {
            // we shall print an error message and continue to the next iteration...
            eprintln!("Invalid line read from the standard input: {}", line);

            continue;
        }

        let index_of_key_separator = index_of_key_separator_option.unwrap();
        // we shall extract key from the received line...
        let key_extraction_result = String::from(&line[..index_of_key_separator]).parse::<u64>();

        if key_extraction_result.is_err() {
            let error = key_extraction_result.unwrap_err();

            // we shall print an error message and continue to the next iteration...
            eprintln!("An error occurred during key extraction: {}", error);

            continue;
        }

        let key = key_extraction_result.unwrap();
        let line_without_key = String::from(&line[index_of_key_separator + KEY_SEPARATOR_LENGTH..]);

        let send_result = sender.send((key, line_without_key));

        if send_result.is_err() {
            let error = send_result.unwrap_err();

            // we shall print an error message...
            eprintln!("An error occurred while sending the received line: {}", error);
        }
    }
}

pub async fn read_line_async(key: u64, receiver: Receiver<(u64, String)>) -> String {
    let mut receiver = receiver.clone();

    while receiver.changed().await.is_ok() {
        let (received_key, received_line) = receiver.borrow().to_owned();

        if !key.eq(&received_key) {
            continue;
        }

        return received_line;
    }

    return String::from("");
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

pub fn start(sender: Sender<(u64, String)>) -> JoinHandle<()> {
    let join_handle = thread::spawn(move || {
        listen(&sender);
    });

    return join_handle;
}
