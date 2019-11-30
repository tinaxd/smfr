
use super::filerw::{SmfReader, SmfError};
use super::super::types::event::*;

type Result<T> = std::result::Result<T, SmfError>;

pub struct SmfParser {
    reader: SmfReader,
    running_status: Option<u8>
}

impl SmfParser {
    pub fn new(reader: SmfReader) -> SmfParser {
        SmfParser{reader, running_status: None}
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

                if let Some(length) = self.reader.next_bytes(4) {

                    match self.parse_mtrk_events() {
                        Ok(pairs) => {
                            let length = ((length[0] as u32) << 3) + ((length[1] as u32) << 2) + ((length[2] as u32) << 1) + (length[3] as u32);
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
                Err(SmfError::new(&format!("invalid midi chunk {:?}", header)))
            }

        } else {
            Err(SmfError::new("midi chunk not found (midi file too short)"))
        }
    }

    pub fn parse_mtrk_events(&mut self) -> Result<Vec<EventPair>> {
        use crate::types::message::MetaEvent::EndOfTrack;

        let mut pairs: Vec<EventPair> = Vec::new();

        loop {
            let pair = self.parse_mtrk_event()?;
            let event = pair.event_copy();
            pairs.push(pair);

            match event {
                MidiEvent::MetaEvent(me) => {
                    match me {
                        EndOfTrack => break,
                        _ => continue,
                    }
                }
                _ => continue,
            }
        }

        Ok(pairs)
    }

    pub fn parse_mtrk_event(&mut self) -> Result<EventPair> {
        let delta_time = self.parse_vlq()?;
        let event_part = self.parse_midi_event()?;
        println!("{:?}", &event_part);
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
                    if byte < 128 {
                        break;
                    }
                },
                None => return Err(SmfError::new(&format!("Error while reading VLQ {:?}", &bin)))
            }
        }
        assert!(bin.len() <= 4, "too long: {}", bin.len());
        Ok(crate::types::message::from_vlq(&bin))
    }

    fn parse_midi_event(&mut self) -> Result<MidiEvent> {
        let first_byte = self.reader.seek_bytes(1).ok_or(SmfError::new("unexpected None"))?[0];
        let midi_event = if 0x80 <= first_byte && first_byte < 0xF0 {// Channel Messages
            MidiEvent::MidiChannelMessage(self.parse_channel_message()?)
        } else if first_byte == 0xFF { // Meta Events
            MidiEvent::MetaEvent(self.parse_meta_event()?)
        } else if first_byte == 0xF0 || first_byte == 0xF7 { // SysEx
            MidiEvent::SysExEvent(self.parse_sysex()?)
        } else {
            MidiEvent::MidiChannelMessage(self.parse_channel_message()?) // Channel Messages with running status
        };

        Ok(midi_event)
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
        println!("{:?}", &three);
        if three == [0xFF, 0x2F, 0x00] {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn parse_channel_message(&mut self) -> Result<crate::types::message::MidiChannelMessage> {
        use crate::types::message::ChannelVoiceMessage::*;
        use crate::types::message::MidiChannelMessage;

        let reader = &mut self.reader;
        let none_msg = SmfError::new("unexpected None");

        let raw_head = reader.seek_bytes(1).ok_or(none_msg.clone())?[0];
        // Check if status byte is omitted (running status)
        let running_status_used =
            raw_head < 0x80;
        let head;
        if running_status_used {
            head = self.running_status.expect("running status is used but no status byte is recorded");
        } else {
            reader.next_bytes(1).unwrap(); // Consume status byte
            head = raw_head;
            self.running_status = Some(head)
        }

        let cvm = match head {
            0x80 ..= 0x8F => {
                let channel = head - 0x80;
                let key = reader.next_bytes(1).ok_or(none_msg.clone())?[0];
                let vel = reader.next_bytes(1).ok_or(none_msg.clone())?[0];
                Some(NoteOff{channel, key, vel})
            },
            0x90 ..= 0x9F => {
                let channel = head - 0x90;
                let key = reader.next_bytes(1).ok_or(none_msg.clone())?[0];
                let vel = reader.next_bytes(1).ok_or(none_msg.clone())?[0];
                Some(NoteOn{channel, key, vel})
            },
            0xA0 ..= 0xAF => {
                let channel = head - 0xA0;
                let key = reader.next_bytes(1).ok_or(none_msg.clone())?[0];
                let vel = reader.next_bytes(1).ok_or(none_msg.clone())?[0];
                Some(NoteOn{channel, key, vel})
            },
            0xB0 ..= 0xBF => { // TODO handling of ChannelModeMessage
                let channel = head - 0xB0;
                let cc = reader.next_bytes(1).ok_or(none_msg.clone())?[0];
                let value = reader.next_bytes(1).ok_or(none_msg.clone())?[0];
                Some(ControlChange{channel, cc, value})
            }
            0xC0 ..= 0xCF => {
                let channel = head - 0xC0;
                let pc = reader.next_bytes(1).ok_or(none_msg.clone())?[0];
                Some(ProgramChange{channel, pc})
            },
            0xD0 ..= 0xDF => {
                let channel = head - 0xD0;
                let vel = reader.next_bytes(1).ok_or(none_msg.clone())?[0];
                Some(ChannelKeyPressure{channel, vel})
            },
            0xE0 ..= 0xEF => {
                let channel = head - 0xE0;
                let msb = reader.next_bytes(1).ok_or(none_msg.clone())?[0];
                let lsb = reader.next_bytes(1).ok_or(none_msg.clone())?[0];
                Some(PitchBend{channel, msb, lsb})
            },
            _ => None
        };
        if cvm.is_some() {
            let cvm = cvm.unwrap();
            return Ok(MidiChannelMessage::ChannelVoiceMessage(cvm));
        }

        // Start parsing ChannelModeMessage
        unimplemented!();
    }

    fn parse_meta_event(&mut self) -> Result<crate::types::message::MetaEvent> {
        use crate::types::message::MetaEvent::*;

        let none_msg = SmfError::new("unexpected None");
        {
            let first = self.reader.next_bytes(1).ok_or(none_msg.clone())?[0];
            assert_eq!(first, 0xFF);
        }
        let meta = self.reader.next_bytes(1).ok_or(none_msg.clone())?[0];
        match meta {
            0x00 => { // Sequence Number
                let length = self.parse_vlq()?;
                assert_eq!(length, 2);
                let data = self.reader.next_bytes(2).ok_or(none_msg.clone())?;
                let number = ((data[0] as u16) << 8) + (data[1] as u16);
                Ok(SequenceNumber{number})
            },

            0x01 ..= 0x09 => { // Text related
                let length = self.parse_vlq()?;
                let text = self.reader.next_bytes(length as usize).ok_or(none_msg.clone())?;
                match meta {
                    0x01 => Ok(TextEvent{length, text}),
                    0x02 => Ok(CopyrightNotice{length, text}),
                    0x03 => Ok(SequenceTrackName{length, text}),
                    0x04 => Ok(InstrumentName{length, text}),
                    0x05 => Ok(Lyric{length, text}),
                    0x06 => Ok(Marker{length, text}),
                    0x07 => Ok(CuePoint{length, text}),
                    _ => unreachable!()
                }
            },

            0x2F => { // End Of Track
                let length = self.parse_vlq()?;
                assert_eq!(length, 0);
                //let _data = self.reader.next_bytes(1).ok_or(SmfError::new("Invalid EndOfTrack"))?;
                Ok(EndOfTrack)
            },

            0x51 => { // Set Tempo
                let length = self.parse_vlq()?;
                assert_eq!(length, 3);
                let data = self.reader.next_bytes(3).ok_or(none_msg.clone())?;
                let tempo = ((data[0] as u32) << 16) + ((data[1] as u32) << 8) + (data[2] as u32);
                Ok(SetTempo{tempo})
            },

            0x54 => { // SMPTE Offset
                let length = self.parse_vlq()?;
                assert_eq!(length, 5);
                let data = self.reader.next_bytes(5).ok_or(none_msg.clone())?;
                let smpte = ((data[0] as u32) << 24) + ((data[1] as u32) << 16) + ((data[2] as u32) << 8) + (data[3] as u32);
                let frame = data[5] as u8;
                Ok(SMPTEOffset{smpte, frame})
            },

            0x58 => { // Time Signature
                let length = self.parse_vlq()?;
                assert_eq!(length, 4);
                let data = self.reader.next_bytes(4).ok_or(none_msg.clone())?;
                Ok(TimeSignature{
                    numerator: data[0],
                    denominator: data[1],
                    clocks: data[2],
                    notes: data[3],
                })
            },

            0x59 => { // Key Signture
                let length = self.parse_vlq()?;
                assert_eq!(length, 2);
                let data = self.reader.next_bytes(2).ok_or(none_msg.clone())?;
                Ok(KeySignature{
                    sf: data[0],
                    minor: data[1],
                })
            },

            0x7F => { // Sequencer-Specific Meta Event
                let length = self.parse_vlq()?;
                let _data = self.reader.next_bytes(length as usize).ok_or(none_msg.clone())?;
                unimplemented!()
            },

            _ => Err(SmfError::new("Unknown meta event"))
        }

    }

    fn parse_sysex(&mut self) -> Result<crate::types::message::SysExEvent> {
        use crate::types::message::SysExEvent;

        match self.reader.next_bytes(1).map(|i| {i[0]}) {
            Some(0xF0) => {
                let length = self.parse_vlq()?;
                let data =  self.reader.next_bytes(length as usize).ok_or(SmfError::new("unexpected None"))?; // data ends with 0xF7
                Ok(SysExEvent::SysExF0{length, data})
            },
            Some(0xF7) => {
                let length = self.parse_vlq()?;
                let data = self.reader.next_bytes(length as usize).ok_or(SmfError::new("unexpected None"))?;
                Ok(SysExEvent::SysExF7{length, data})
            },
            _ => Err(SmfError::new("error parsing SysEx"))
        }
    }
}