
use super::message;
use super::message::SmfElement;

pub enum MidiEvent {
    MidiChannelMessage{message: message::MidiChannelMessage},
    MetaEvent{event: message::MetaEvent},
    SysExEvent{event: message::SysExEvent}
}

/// Represents a delta_time-event pair in SMF
pub struct EventPair {
    time: u32, /// can be used as either delta time or absolute time
    event: MidiEvent
}

impl SmfElement for MidiEvent {
    fn raw(&self) -> std::vec::Vec<u8> {
        use MidiEvent::*;
        match self {
            MidiChannelMessage{message} => message.raw(),
            MetaEvent{event} => event.raw(),
            SysExEvent{event} => event.raw()
        }
    }
}

impl SmfElement for EventPair {
    /// Works correctly only when EventPair::time is delta time
    fn raw(&self) -> std::vec::Vec<u8> {
        let vec_of_time = message::to_vlq(self.time);
        let mut ret = vec_of_time;
        ret.extend(self.event.raw());
        ret
    }
}

// MThd and MTrk //

pub enum MidiChunk {
    HeaderChunk{chunk: HeaderChunk},
    TrackChunk{chunk: TrackChunk}
}

pub struct HeaderChunk {
    length: u32,
    format: u16,
    tracks: u16,
    division: u16
}

pub struct TrackChunk {
    length: u32,
    data: std::vec::Vec<EventPair>
}

impl SmfElement for HeaderChunk {
    fn raw(&self) -> std::vec::Vec<u8> {
        vec![
            'M' as u8, 'T' as u8, 'h' as u8, 'd' as u8,
            (self.length & (0xFF << 3)) as u8, (self.length & (0xFF << 2)) as u8 , (self.length & (0xFF << 1)) as u8, (self.length & 0xFF) as u8,
            (self.format & (0xFF << 1)) as u8, (self.format & 0xFF) as u8,
            (self.tracks & (0xFF << 1)) as u8, (self.tracks & 0xFF) as u8,
            (self.division & (0xFF << 1)) as u8, (self.division & 0xFF) as u8
        ]
    }
}

impl SmfElement for TrackChunk {
    fn raw(&self) -> std::vec::Vec<u8> {
        let mut binary = vec![
            'M' as u8, 'T' as u8, 'r' as u8, 'k' as u8,
            (self.length & (0xFF << 3)) as u8, (self.length & (0xFF << 2)) as u8 , (self.length & (0xFF << 1)) as u8, (self.length & 0xFF) as u8,
        ];
        for pair in &self.data {
            binary.extend(pair.raw());
        }
        binary
    }
}
