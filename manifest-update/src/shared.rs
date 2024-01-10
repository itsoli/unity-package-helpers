use std::error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::result;

use unicode_bom::Bom;

pub(crate) type Result<T> = result::Result<T, Box<dyn error::Error>>;

pub(crate) fn open_reader<P: AsRef<Path>>(path: P) -> Result<BufReader::<File>> {
    let mut file = File::open(&path)?;
    let bom_len = Bom::from(&mut file).len();
    file = File::open(&path)?;
    let mut reader = BufReader::new(file);
    if bom_len > 0 {
        reader.seek_relative(bom_len as i64)?;
    }
    Ok(reader)
}
