
use super::message;
use super::message::SmfElement;

#[derive(Debug, Clone)]
pub enum MidiEvent {
    MidiChannelMessage(message::MidiChannelMessage),
    MetaEvent(message::MetaEvent),
    SysExEvent(message::SysExEvent)
}

/// Represents a delta_time-event pair in SMF
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum MidiChunk {
    HeaderChunk(HeaderChunk),
    TrackChunk(TrackChunk)
}

#[derive(Debug, Clone)]
pub struct HeaderChunk {
    pub length: u32, // should always be 0x00000006
    pub format: u16,
    pub tracks: u16,
    pub resolution: u16
}

#[derive(Debug, Clone)]
pub struct TrackChunk {
    pub length: u32,
    pub events: std::vec::Vec<EventPair>,
}

pub struct TrackChunkIter<'a> {
    next: usize,
    target: &'a TrackChunk
}

impl SmfElement for HeaderChunk {
    fn raw(&self) -> std::vec::Vec<u8> {
        vec![
            'M' as u8, 'T' as u8, 'h' as u8, 'd' as u8,
            ((self.length & (0xFF << 24)) >> 24) as u8, ((self.length & (0xFF << 16)) >> 16) as u8 , ((self.length & (0xFF << 8)) >> 8) as u8, (self.length & 0xFF) as u8,
            ((self.format & (0xFF << 8)) >> 8) as u8, (self.format & 0xFF) as u8,
            ((self.tracks & (0xFF << 8)) >> 8) as u8, (self.tracks & 0xFF) as u8,
            ((self.resolution & (0xFF << 8)) >> 8) as u8, (self.resolution & 0xFF) as u8
        ]
    }
}

impl SmfElement for TrackChunk {
    fn raw(&self) -> std::vec::Vec<u8> {
        let mut binary = vec![
            'M' as u8, 'T' as u8, 'r' as u8, 'k' as u8,
            ((self.length & (0xFF << 24)) >> 24) as u8, ((self.length & (0xFF << 16)) >> 16) as u8 , ((self.length & (0xFF << 8)) >> 8) as u8, (self.length & 0xFF) as u8,
        ];
        for pair in &self.events {
            binary.extend(pair.raw());
        }
        binary
    }
}

impl std::ops::Index<usize> for TrackChunk {
    type Output = EventPair;
    fn index(&self, index: usize) -> &Self::Output {
        &self.events[index]
    }
}

impl<'a> Iterator for TrackChunkIter<'a> {
    type Item = &'a EventPair;

    fn next(&mut self) -> Option<Self::Item> {
        self.next += 1;
        self.target.events.get(self.next - 1)
    }
}

impl<'a> IntoIterator for &'a TrackChunk {
    type Item = &'a EventPair;
    type IntoIter = TrackChunkIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TrackChunkIter {next: 0, target: self}
    }
}

impl<'a> ExactSizeIterator for TrackChunkIter<'a> {
    fn len(&self) -> usize {
        self.target.events.len()
    }
}

impl TrackChunk {
    pub fn len(&self) -> usize {
        self.events.len()
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

// MidiFile //

#[derive(Debug, Clone)]
pub struct SMF {
    header: HeaderChunk,
    tracks: Vec<TrackChunk>
}

impl SmfElement for SMF {
    fn raw(&self) -> Vec<u8> {
        let mut raw_all = self.header.raw();

        for track in &self.tracks {
            raw_all.extend(track.raw());
        }

        raw_all
    }
}

impl SMF {
    pub fn new(header: HeaderChunk, tracks: Vec<TrackChunk>) -> SMF{
        SMF { header, tracks, }
    }

    /// Only works when time is absolute.
    pub fn merge_tracks(&mut self) {
        let mut merged: Vec<EventPair> = Vec::new();

        let num_events: Vec<usize> = self.tracks.iter().map(|i| i.len()).collect();
        let mut indices: Vec<usize> = vec![0; self.tracks.len()];
        loop {
            let mut min_time = std::u32::MAX;
            let mut min_index = std::usize::MAX;
            for (i, tracks) in self.tracks.iter().enumerate() {
                if indices[i] >= num_events[i] {
                    continue;
                }

                let time = tracks[indices[i]].time;
                if time < min_time {
                    min_time = time;
                    min_index = i;
                }
            }

            if min_index == std::usize::MAX {
                break;
            }

            let pair = self.tracks[min_index][indices[min_index]].clone();
            indices[min_index] += 1;

            merged.push(pair);
        }

        let sum_length = self.tracks.iter().fold(0, |acc, track| acc + track.length);

        self.tracks = vec![TrackChunk {
            length: sum_length,
            events: merged,
        }];
    }
}