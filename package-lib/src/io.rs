use std::fs::File;
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

use serde::{de, Serialize};
use unicode_bom::Bom;

use crate::Result;

/// Trims whitespace from the beginning and end of the string.
pub fn trim_string(input: &mut String) {
    while let Some(ch) = input.chars().next_back() {
        if !ch.is_whitespace() {
            break;
        }
        input.pop();
    }
    while let Some(ch) = input.chars().next() {
        if !ch.is_whitespace() {
            break;
        }
        input.remove(0);
    }
}

/// Attamps to open a file read-only and skips the BOM if present.
pub fn open_file_skip_bom<P: AsRef<Path>>(path: P) -> Result<File> {
    let mut file = File::open(path)?;
    let bom_len = Bom::from(&mut file).len();
    file.seek(SeekFrom::Start(bom_len as u64))?;
    Ok(file)
}

/// Attempts to read a file into a string and skips the BOM if present.
pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut buffer = String::new();
    open_file_skip_bom(path)?.read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// Attempts to read a file as JSON and skips the BOM if present.
pub fn read_json<P, T>(path: P) -> Result<T>
where
    P: AsRef<Path>,
    T: de::DeserializeOwned,
{
    let mut buffer = String::new();
    open_file_skip_bom(path)?.read_to_string(&mut buffer)?;
    Ok(serde_json::from_str(&buffer)?)
}

/// Attempts to write a JSON value to a file.
pub fn write_json<P, T>(path: P, data: &T) -> Result<()>
where
    P: AsRef<Path>,
    T: ?Sized + Serialize,
{
    let mut writer = BufWriter::new(File::create(path)?);
    serde_json::to_writer_pretty(&mut writer, data)?;
    let _ = writer.write(b"\n")?;
    writer.flush()?;
    Ok(())
}
