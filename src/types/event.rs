
use super::message;
use super::message::SmfElement;

#[derive(Debug, Clone)]
pub enum MidiEvent {
    MidiChannelMessage(message::MidiChannelMessage),
    MetaEvent(message::MetaEvent),
    SysExEvent(message::SysExEvent)
}

/// Represents a delta_time-event pair in SMF
#[derive(Debug)]
pub struct EventPair {
    time: u32, /// can be used as either delta time or absolute time
    event: MidiEvent
}

impl SmfElement for MidiEvent {
    fn raw(&self) -> std::vec::Vec<u8> {
        use MidiEvent::*;
        match self {
            MidiChannelMessage(message) => message.raw(),
            MetaEvent(event) => event.raw(),
            SysExEvent(event) => event.raw()
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

impl EventPair {
    pub fn new(time: u32, event: MidiEvent) -> EventPair {
        EventPair {time, event, }
    }

    pub fn event_copy(&self) -> MidiEvent {
        self.event.clone()
    }

    pub fn event(&self) -> &MidiEvent {
        &self.event
    }
}

// MThd and MTrk //

#[derive(Debug)]
pub enum MidiChunk {
    HeaderChunk(HeaderChunk),
    TrackChunk(TrackChunk)
}

#[derive(Debug)]
pub struct HeaderChunk {
    pub length: u32, // should always be 0x00000006
    pub format: u16,
    pub tracks: u16,
    pub resolution: u16
}

#[derive(Debug)]
pub struct TrackChunk {
    pub length: u32,
    pub events: std::vec::Vec<EventPair>
}

impl SmfElement for HeaderChunk {
    fn raw(&self) -> std::vec::Vec<u8> {
        vec![
            'M' as u8, 'T' as u8, 'h' as u8, 'd' as u8,
            (self.length & (0xFF << 3)) as u8, (self.length & (0xFF << 2)) as u8 , (self.length & (0xFF << 1)) as u8, (self.length & 0xFF) as u8,
            (self.format & (0xFF << 1)) as u8, (self.format & 0xFF) as u8,
            (self.tracks & (0xFF << 1)) as u8, (self.tracks & 0xFF) as u8,
            (self.resolution & (0xFF << 1)) as u8, (self.resolution & 0xFF) as u8
        ]
    }
}

impl SmfElement for TrackChunk {
    fn raw(&self) -> std::vec::Vec<u8> {
        let mut binary = vec![
            'M' as u8, 'T' as u8, 'r' as u8, 'k' as u8,
            (self.length & (0xFF << 3)) as u8, (self.length & (0xFF << 2)) as u8 , (self.length & (0xFF << 1)) as u8, (self.length & 0xFF) as u8,
        ];
        for pair in &self.events {
            binary.extend(pair.raw());
        }
        binary
    }
}

impl SmfElement for MidiChunk {
    fn raw(&self) -> Vec<u8> {
        use MidiChunk::*;
        match self {
            HeaderChunk(x) => x.raw(),
            TrackChunk(x) => x.raw()
        }
    }
}