
// SMF Error //

#[derive(Debug)]
pub struct SmfError {
    message: String
}

impl Clone for SmfError {
    fn clone(&self) -> Self {
        SmfError {
            message: self.message.clone()
        }
    }
}

impl SmfError {
    pub fn new(message: &str) -> SmfError {
        SmfError{message: message.to_string()}
    }
}

impl std::error::Error for SmfError {}

impl std::fmt::Display for SmfError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

////////////////

type Result<T> = std::result::Result<T, SmfError>;

pub struct SmfReader {
    data:    Vec<u8>,
    pointer: usize
}

impl SmfReader {
    pub fn reset_pointer(&mut self) {
        self.pointer = 0;
    }

    pub fn read_from_file(filepath: &std::path::Path) -> Result<SmfReader> {
        use std::fs::File;
        use std::io::Read;

        if !filepath.exists() {
            return Err(SmfError{message: "File not found".to_string()})
        }
        
        if let Ok(mut file) = File::open(filepath) {
            let mut buf: Vec<u8> = Vec::new();
            match file.read_to_end(&mut buf) {
                Ok(_) => Ok(SmfReader{data: buf, pointer: 0}),
                Err(e) => Err(SmfError{message: e.to_string()})
            }
        } else {
            Err(SmfError{message: "File IO Error".to_string()})
        }
    }

    pub fn next_bytes_ref(&mut self, bytes: usize) -> Option<&[u8]> {
        if self.pointer + bytes > self.data.len() {
            return None;
        }
        let ret = &self.data[self.pointer .. self.pointer + bytes];
        self.pointer += bytes;
        Some(ret)
    }

    pub fn next_bytes(&mut self, bytes: usize) -> Option<Vec<u8>> {
        match self.next_bytes_ref(bytes) {
            Some(x) => Some(x.to_vec()),
            None => None
        }
    }

    pub fn seek_bytes_ref(&self, bytes: usize) -> Option<&[u8]> {
        if self.pointer + bytes >= self.data.len() {
            return None;
        }
        Some(&self.data[self.pointer .. self.pointer + bytes])
    }

    pub fn seek_bytes(&self, bytes: usize) -> Option<Vec<u8>> {
        match self.seek_bytes_ref(bytes) {
            Some(x) => Some(x.to_vec()),
            None => None
        }
    }
}

pub fn write_to_file(filepath: &std::path::Path, smf: &crate::types::event::SMF, overwrite: bool) -> std::io::Result<()> {
    use std::fs::OpenOptions;
    use crate::types::message::SmfElement;
    use std::io::BufWriter;
    use std::io::Write;

    if !overwrite && filepath.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Overwrite is disabled and file already exists"))
    }
    let file = OpenOptions::new().write(true).truncate(true).create(true).open(filepath)?;
    let mut file = BufWriter::new(file);

    let mut smf = smf.clone();
    smf.recalculate_length();
    let binary = smf.raw();

    file.write(&binary)?;
    Ok(())
}