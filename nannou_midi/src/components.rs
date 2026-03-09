use bevy::prelude::*;

#[derive(Component, Clone, Debug, Reflect)]
pub struct MidiPort {
    pub direction: MidiPortDirection,
}

#[derive(Clone, Debug, PartialEq, Eq, Reflect)]
pub enum MidiPortDirection {
    Input,
    Output,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct MidiInput {
    pub port: Option<Entity>,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct MidiOutput {
    pub port: Option<Entity>,
}

#[derive(Component)]
pub struct MidiInputStream {
    pub(crate) messages: Vec<MidiData>,
}

impl MidiInputStream {
    pub(crate) fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn read(&mut self) -> impl Iterator<Item = MidiData> + '_ {
        self.messages.drain(..)
    }
}

#[derive(Component)]
pub struct MidiOutputStream {
    pub(crate) outbox: Vec<MidiMessage>,
}

impl MidiOutputStream {
    pub(crate) fn new() -> Self {
        Self { outbox: Vec::new() }
    }

    pub fn send(&mut self, msg: MidiMessage) {
        self.outbox.push(msg);
    }
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct MidiError {
    pub message: String,
}

const NOTE_ON_STATUS: u8 = 0b1001_0000;
const NOTE_OFF_STATUS: u8 = 0b1000_0000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MidiMessage {
    pub msg: [u8; 3],
}

impl From<[u8; 3]> for MidiMessage {
    fn from(msg: [u8; 3]) -> Self {
        MidiMessage { msg }
    }
}

impl MidiMessage {
    #[must_use]
    pub fn is_note_on(&self) -> bool {
        (self.msg[0] & 0xF0) == NOTE_ON_STATUS && self.msg[2] != 0
    }

    #[must_use]
    pub fn is_note_off(&self) -> bool {
        (self.msg[0] & 0xF0) == NOTE_OFF_STATUS
            || ((self.msg[0] & 0xF0) == NOTE_ON_STATUS && self.msg[2] == 0)
    }

    #[must_use]
    pub fn channel(&self) -> u8 {
        self.msg[0] & 0x0F
    }
}

#[derive(Clone, Debug)]
pub struct MidiData {
    pub stamp: u64,
    pub message: MidiMessage,
}
