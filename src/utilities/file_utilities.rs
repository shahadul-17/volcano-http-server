use std::{io::{self, BufReader}, fs::{self, File}};

const BUFFER_LENGTH_IN_BYTES: usize = 8192;         // 8 KB...

pub fn create_buffered_file_reader(file_path: String) -> io::Result<BufReader<File>> {
    let file = File::open(file_path)?;
    let buffered_reader = BufReader::with_capacity(BUFFER_LENGTH_IN_BYTES, file);

    return Ok(buffered_reader);
}

pub fn exists(file_path: String) -> bool {
    let metadata_result = fs::metadata(file_path);

    if metadata_result.is_err() { return false; }

    let metadata = metadata_result.unwrap();

    // we shall return true if and only if the path belongs to a file...
    return metadata.is_file();
}
