use std::{io::{self, BufReader}, fs::File};

const BUFFER_LENGTH_IN_BYTES: usize = 8192;         // 8 KB...

pub fn create_buffered_file_reader(file_path: String) -> io::Result<BufReader<File>> {
    let file = File::open(file_path)?;
    let buffered_reader = BufReader::with_capacity(BUFFER_LENGTH_IN_BYTES, file);

    return Ok(buffered_reader);
}
