
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
        let mut pairs: Vec<EventPair> = Vec::new();

        while !self.seek_if_track_end()? {
            let pair = self.parse_mtrk_event()?;
            pairs.push(pair);
        }
        // skip EndOfTrack (3 bytes)
        self.reader.next_bytes(3);

        Ok(pairs)
    }

    pub fn parse_mtrk_event(&mut self) -> Result<EventPair> {
        let delta_time = self.parse_vlq()?;
        let event_part = self.parse_midi_event()?;
        Ok(EventPair::new(
            delta_time,
            event_part
        ))
    }

    fn parse_vlq(&mut self) -> Result<u32> {
        let mut bin: Vec<u8> = Vec::new();
        loop {
            match self.reader.next_bytes(1) {
                Some(bytes) => {
                    let byte = bytes[0];
                    bin.push(byte);
                    if byte > 128 {
                        break;
                    }
                },
                None => return Err(SmfError::new("Error while reading VLQ"))
            }
        }
        Ok(crate::types::message::from_vlq(&bin))
    }

    fn parse_midi_event(&mut self) -> Result<MidiEvent> {
        let first_byte = self.reader.seek_bytes(1).ok_or(SmfError::new("unexpected None"))?.get(0).unwrap();
        if &0x80 <= first_byte && first_byte < &0xF0 {// Channel Messages
            self.parse_channel_message()
        } else if first_byte == &0xFF { // Meta Events
            self.parse_meta_event()
        } else if first_byte == &0xF0 || first_byte == &0xF7 { // SysEx
            self.parse_sysex()
        } else {
            unimplemented!()
        }
    }

    /// Returns true if the next token is EndOfTrack.
    /// This method does not increment SmfReader pointer.
    fn seek_if_track_end(&self) -> Result<bool> {
        // EndOfTrack FF 2F 00
        let three = self.reader.seek_bytes_ref(3);
        if three.is_none() {
            return Err(SmfError::new("Failed to seek EndOfTrack"))
        }

        let three = three.unwrap();
        if three == [0xFF, 0x2F, 0x00] {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn parse_channel_message(&mut self) -> Result<crate::types::message::MidiChannelMessage> {
        unimplemented!()
    }

    fn parse_meta_event(&mut self) -> Result<crate::types::message::MetaEvent> {
        unimplemented!()
    }

    fn parse_sysex(&mut self) -> Result<crate::types::message::SysExEvent> {
        unimplemented!()
    }
}