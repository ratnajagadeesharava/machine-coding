//! Aggregation.
#![allow(unused_variables)]

pub mod contract;

use contract::MusicError;
use std::rc::Rc;

/// A song.
pub struct Song;

impl Song {
    pub fn new(title: &str, seconds: u32) -> Result<Song, MusicError> {
        todo!()
    }

    pub fn title(&self) -> String {
        todo!()
    }

    pub fn seconds(&self) -> u32 {
        todo!()
    }

    pub fn rename(&self, new_title: &str) -> Result<(), MusicError> {
        todo!()
    }
}

/// A playlist of songs.
pub struct Playlist;

impl Playlist {
    pub fn new(name: &str) -> Playlist {
        todo!()
    }

    pub fn name(&self) -> &str {
        todo!()
    }

    pub fn add(&mut self, song: Rc<Song>) -> bool {
        todo!()
    }

    pub fn remove(&mut self, song: &Rc<Song>) -> bool {
        todo!()
    }

    pub fn songs(&self) -> &[Rc<Song>] {
        todo!()
    }

    pub fn total_seconds(&self) -> u32 {
        todo!()
    }

    pub fn clear(&mut self) {
        todo!()
    }
}
