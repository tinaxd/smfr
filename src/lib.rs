pub mod types;
pub mod file;

#[cfg(test)]
mod tests {
    use super::types::message;
    use super::file::{filerw, parser};

    #[test]
    fn from_vlq_1() {
        let bin = vec![0b01111111];
        assert_eq!(message::from_vlq(&bin), 127);
    }

    #[test]
    fn from_vlq_2() {
        let bin = vec![0b11111111, 0b01111111];
        assert_eq!(message::from_vlq(&bin), 16383);
    }

    #[test]
    fn to_vlq_1() {
        let num = 126;
        let expected = vec![0b01111110];
        assert_eq!(message::to_vlq(num), expected);
    }

    #[test]
    fn to_vlq_2() {
        let num = 100000;
        let expected = vec![0b10000110, 0b10001101, 0b00100000];
        assert_eq!(message::to_vlq(num), expected);
    }

    #[test]
    fn to_vlq_3() {
        let num = 1000;
        let expected = vec![0b10000111, 0b01101000];
        assert_eq!(message::to_vlq(num), expected);
    }

    #[test]
    fn midi_parse_mthd() {
        use std::path::Path;

        println!("Current directory {:?}", std::env::current_dir().unwrap());
        let reader = filerw::SmfReader::read_from_file(Path::new("test.mid"));
        match reader {
            Ok(r) => {
                let mut parser = parser::SmfParser::new(r);
                match parser.next_chunk() { // Read the first MThd chunk
                    Ok(chunk) => {
                        match chunk {
                            super::types::event::MidiChunk::HeaderChunk(chunk) => {
                                assert_eq!(chunk.length, 6);
                                assert_eq!(chunk.tracks, 6);
                                assert_eq!(chunk.format, 1);
                                assert_eq!(chunk.resolution, 226);
                            },
                            _ => {
                                panic!("Not MThd chunk!");
                            }
                        }
                    },
                    Err(e) => {
                        panic!("{}", e);
                    }
                }
            },

            Err(e) => {
                panic!("{}", e);
            }
        }

    }

    #[test]
    #[ignore]
    fn midi_parse_all() {
        use std::path::Path;

        println!("Current directory {:?}", std::env::current_dir().unwrap());
        let reader = filerw::SmfReader::read_from_file(Path::new("test.mid"));
        match reader {
            Ok(r) => {
                let mut parser = parser::SmfParser::new(r);
                loop {
                    match parser.next_chunk() {
                        Ok(chunk) => {
                            match chunk {
                                super::types::event::MidiChunk::HeaderChunk(chunk) => {
                                    println!("{:?}", chunk)
                                },
                                super::types::event::MidiChunk::TrackChunk(chunk) => {
                                    println!("{:?}", chunk)
                                }
                            }
                        },
                        Err(e) => {
                            panic!("{}", e);
                        }
                    }
                }
            },

            Err(e) => {
                panic!("{}", e);
            }
        }
    }
}
