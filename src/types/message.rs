
pub fn to_vlq(val: u32) -> std::vec::Vec<u8> {
    let mut reverse_binary = std::vec::Vec::new();
    let mut rem = val;
    while rem != 0 {
        let seven = rem & 0b01111111;
        rem = rem >> 7;
        reverse_binary.push(seven as u8 + 128);
    }
    reverse_binary[0] -= 128;
    reverse_binary.reverse();
    reverse_binary
}

pub fn from_vlq(binary: &std::vec::Vec<u8>) -> u32 {
    let mut ret: u32 = 0;
    for (i, bin) in binary.iter().enumerate() {
        let num =
            if *bin < 128 {
                *bin as u32
            } else {
                (bin - 128) as u32
            };
        ret += num << (7 * (binary.len() - i - 1));
    }
    ret
}

pub trait SmfElement {
    /// Returns binary expressions of midi data
    fn raw(&self) -> std::vec::Vec<u8>;
}

// ChannelVoiceMessage and ChannelModeMessage //

pub enum MidiChannelMessage {
    ChannelVoiceMessage{message: ChannelVoiceMessage},
    ChannelModeMessage{message: ChannelModeMessage},
}

pub enum ChannelVoiceMessage {
    NoteOff{channel: u8, key: u8, vel: u8},
    NoteOn{channel: u8, key: u8, vel: u8},
    PolyphonicKeyPressure{channel: u8, key: u8, vel: u8},
    ControllerChange{channel: u8, cc: u8, value: u8},
    ProgramChange{channel: u8, vel: u8},
    ChannelKeyPressure{channel: u8, vel: u8},
    PitchBend{channel: u8, lsb: u8, msb: u8}
}

pub enum ChannelModeMessage {
    AllSoundOff{channel: u8},
    ResetAllControllers{channel: u8},
    AllNotesOff{channel: u8}
}

impl ChannelVoiceMessage {
    pub fn channel(&self) -> u8 {
        let ch = match self {
            ChannelVoiceMessage::NoteOff{channel, ..} => channel,
            ChannelVoiceMessage::NoteOn{channel, ..} => channel,
            ChannelVoiceMessage::PolyphonicKeyPressure{channel, ..} => channel,
            ChannelVoiceMessage::ControllerChange{channel, ..} => channel,
            ChannelVoiceMessage::ProgramChange{channel, ..} => channel,
            ChannelVoiceMessage::ChannelKeyPressure{channel, ..} => channel,
            ChannelVoiceMessage::PitchBend{channel, ..} => channel
        };
        *ch
    }

    pub fn status_byte(&self) -> u8 {
        match self {
            ChannelVoiceMessage::NoteOff{channel, ..} => 0x80 + channel,
            ChannelVoiceMessage::NoteOn{channel, ..} => 0x90 + channel,
            ChannelVoiceMessage::PolyphonicKeyPressure{channel, ..} => 0xA0 + channel,
            ChannelVoiceMessage::ControllerChange{channel, ..} => 0xB0 + channel,
            ChannelVoiceMessage::ProgramChange{channel, ..} => 0xC0 + channel,
            ChannelVoiceMessage::ChannelKeyPressure{channel, ..} => 0xD0 + channel,
            ChannelVoiceMessage::PitchBend{channel, ..} => 0xE0 + channel
        }
    }
}

impl ChannelModeMessage {
    pub fn channel(&self) -> u8 {
        let ch = match self {
            ChannelModeMessage::AllSoundOff{channel} => channel,
            ChannelModeMessage::ResetAllControllers{channel} => channel,
            ChannelModeMessage::AllNotesOff{channel} => channel
        };
        *ch
    }

    pub fn status_byte(&self) -> u8 {
        0xB0 + self.channel()
    }
}

impl MidiChannelMessage {
    pub fn channel(&self) -> u8 {
        match self {
            MidiChannelMessage::ChannelVoiceMessage{message} => message.channel(),
            MidiChannelMessage::ChannelModeMessage{message} => message.channel()
        }
    }
}

impl SmfElement for ChannelVoiceMessage {
    fn raw(&self) -> std::vec::Vec<u8> {
        use ChannelVoiceMessage::*;
        match *self {
            NoteOn{key, vel, ..} => vec![self.status_byte(), key, vel],
            NoteOff{key, vel, ..} => vec![self.status_byte(), key, vel],
            PolyphonicKeyPressure{key, vel, ..} => vec![self.status_byte(), key, vel],
            ControllerChange{cc, value, ..} => vec![self.status_byte(), cc, value],
            ProgramChange{vel, ..} => vec![self.status_byte(), vel],
            ChannelKeyPressure{vel, ..} => vec![self.status_byte(), vel],
            PitchBend{lsb, msb, ..} => vec![self.status_byte(), lsb, msb]
        }
    }
}

impl SmfElement for ChannelModeMessage {
    fn raw(&self) -> std::vec::Vec<u8> {
        use ChannelModeMessage::*;
        match *self {
            AllSoundOff{..} => vec![self.status_byte(), 0x78, 0],
            ResetAllControllers{..} => vec![self.status_byte(), 0x79, 0],
            AllNotesOff{..} => vec![self.status_byte(), 0x7B, 0]
        }
    }
}

impl SmfElement for MidiChannelMessage {
    fn raw(&self) -> std::vec::Vec<u8> {
        match self {
            MidiChannelMessage::ChannelVoiceMessage{message} => message.raw(),
            MidiChannelMessage::ChannelModeMessage{message} => message.raw()
        }
    }
}

// Meta Events // 

pub enum MetaEvent {
    SequenceNumber{number: u16},
    TextEvent{length: u32, text: std::vec::Vec<u8>},
    CopyrightNotice{length: u32, text: std::vec::Vec<u8>},
    SequenceTrackName{length: u32, text: std::vec::Vec<u8>},
    InstrumentName{length: u32, text: std::vec::Vec<u8>},
    Lyric{length: u32, text: std::vec::Vec<u8>},
    Marker{length: u32, text: std::vec::Vec<u8>},
    CuePoint{length: u32, text: std::vec::Vec<u8>},
    MIDIChannelPrefix{channel: u8},
    EndOfTrack,
    SetTempo{tempo: u32},
    SMTPEOffset{smtpe: u32, frame: u8},
    TimeSignature{numerator: u8, denominator: u8, clocks: u8, notes: u8},
    KeySignature{sf: u8, minor: u8},
    SequencerSpecific{length: u32, id: u8, data: u8}
}

impl SmfElement for MetaEvent {
    fn raw(&self) -> std::vec::Vec<u8> {
        fn _helper(code: u8, len: u32, text: &std::vec::Vec<u8>) -> std::vec::Vec<u8> {
            let mut ret = vec![0xff, code];
    
            let vec_of_len = to_vlq(len);
            ret.extend(vec_of_len);
            ret.extend(text);
            ret
        }

        use MetaEvent::*;
        match self {
            SequenceNumber{number} => vec![0xff, 0x00, 0x02, (number & 0b1100) as u8, (number & 0b0011) as u8],
            TextEvent{length, text} => _helper(0x01, *length, text),
            CopyrightNotice{length, text} => _helper(0x02, *length, text),
            SequenceTrackName{length, text} => _helper(0x03, *length, text),
            InstrumentName{length, text} => _helper(0x04, *length, text),
            Lyric{length, text} => _helper(0x05, *length, text),
            Marker{length, text} => _helper(0x06, *length, text),
            CuePoint{length, text} => _helper(0x07, *length, text),
            MIDIChannelPrefix{channel} => vec![0xff, 0x20, 0x01, *channel],
            EndOfTrack => vec![0xff, 0x2f, 0x00],
            SetTempo{tempo} => vec![0xff, 0x51, 0x03, (tempo & 0b110000) as u8, (tempo & 0b001100) as u8, (tempo & 0b000011) as u8],
            SMTPEOffset{smtpe, frame} => vec![0xff, 0x54 ,0x05, (smtpe & 0b11000000) as u8, (smtpe & 0b00110000) as u8, (smtpe & 0b00001100) as u8, (smtpe & 0b00000011) as u8, *frame],
            TimeSignature{numerator, denominator, clocks, notes} => vec![0xff, 0x58, 0x04, *numerator, *denominator, *clocks, *notes],
            KeySignature{sf, minor} => vec![0xff, 0x59, 0x02, *sf, *minor],
            SequencerSpecific{length, id, data} => {
                let mut ret = vec![0xff, 0x7f];
                let vec_of_len = to_vlq(*length);
                ret.extend(vec_of_len);
                ret.extend(vec![id, data]);
                ret
            }
        }
    }
}

// System Exclusive Events //
pub enum SysExEvent {
    SysExF0{length: u32, data: std::vec::Vec<u8>},
    SysExF7{length: u32, data: std::vec::Vec<u8>}
}

impl SmfElement for SysExEvent {
    fn raw(&self) -> std::vec::Vec<u8> {
        use SysExEvent::*;
        fn _helper(prefix: u8, len: u32, data: &std::vec::Vec<u8>) -> std::vec::Vec<u8> {
            let mut ret = vec![prefix];
            let vec_of_len = to_vlq(len);
            ret.extend(vec_of_len);
            ret.extend(data);
            ret
        }

        match self {
            SysExF0{length, data} => _helper(0xF0, *length, data),
            SysExF7{length, data} => _helper(0xF7, *length, data)
        }
    }
}