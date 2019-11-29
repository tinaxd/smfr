
// SMF Error //

#[derive(Debug)]
pub struct SmfError {
    message: String
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
        if self.pointer + bytes >= self.data.len() {
            return None;
        }
        let ret = &self.data[self.pointer..self.pointer + bytes];
        self.pointer += bytes;
        Some(ret)
    }

    pub fn next_bytes(&mut self, bytes: usize) -> Option<Vec<u8>> {
        match self.next_bytes_ref(bytes) {
            Some(x) => Some(x.to_vec()),
            None => None
        }
    }
}