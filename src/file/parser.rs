
use super::filerw::{SmfReader, SmfError};
use super::super::types::event::*;

type Result<T> = std::result::Result<T, SmfError>;

pub struct SmfParser {
    reader: SmfReader,
}

impl SmfParser {
    pub fn new(reader: SmfReader) -> SmfParser {
        SmfParser{reader}
    }

    pub fn next_chunk(&mut self) -> Result<MidiChunk> {
        if let Some(header) = self.reader.next_bytes(4) {

            if header == &vec!['M' as u8, 'T' as u8, 'h' as u8, 'd' as u8][..] {
                // MThd length format tracks timebase

                let length     = self.reader.next_bytes(4);
                let format     = self.reader.next_bytes(2);
                let tracks     = self.reader.next_bytes(2);
                let resolution = self.reader.next_bytes(2);

                if vec![&length, &format, &tracks, &resolution].iter().all(|x| x.is_some()) {
                    let length     = length.unwrap();
                    let format     = format.unwrap();
                    let tracks     = tracks.unwrap();
                    let resolution = resolution.unwrap();
                    Ok(MidiChunk::HeaderChunk(HeaderChunk{
                        length:     ((length[0] as u32) << 24) + ((length[1] as u32) << 16) + ((length[2] as u32) << 8) + (length[3] as u32),
                        format:     ((format[0] as u16) << 8) + (format[1] as u16),
                        tracks:     ((tracks[0] as u16) << 8) + (tracks[1] as u16),
                        resolution: ((resolution[0] as u16) << 1) + (resolution[1] as u16)
                    }))
                } else {
                    Err(SmfError::new("invalid MThd chunk info"))
                }
                

            } else if header == &vec!['M' as u8, 'T' as u8, 'r' as u8, 'k' as u8][..] {
                // MTrk length mtrk_events

                if let Some(length) = self.reader.next_bytes(8) {

                    match self.parse_mtrk_events() {
                        Ok(pairs) => {
                            let length = (length[0] << 3 + length[1] << 2 + length[2] << 1 + length[3]).into();
                            Ok(MidiChunk::TrackChunk(TrackChunk{
                                length,
                                events: pairs
                            }))
                        },
                        Err(e) => Err(e)
                    }

                } else {
                    Err(SmfError::new("invalid MTrk length"))
                }

            } else {
                Err(SmfError::new("invalid midi chunk"))
            }

        } else {
            Err(SmfError::new("midi chunk not found (midi file too short)"))
        }
    }

    pub fn parse_mtrk_events(&mut self) -> Result<Vec<EventPair>> {
        unimplemented!()
    }
}