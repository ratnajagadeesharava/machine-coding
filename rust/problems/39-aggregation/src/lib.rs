//! Aggregation — a `Playlist` holds shared references (`Rc<Song>`) to songs it does not own.
#![allow(unused_variables)]

pub mod contract;

use contract::MusicError;
use std::rc::Rc;

/// A song exists on its own. `rename` takes `&self`, so you will need interior mutability.
pub struct Song;

impl Song {
    /// Blank title -> `InvalidTitle`; zero seconds -> `InvalidDuration`.
    pub fn new(title: &str, seconds: u32) -> Result<Song, MusicError> {
        todo!()
    }

    pub fn title(&self) -> String {
        todo!()
    }

    pub fn seconds(&self) -> u32 {
        todo!()
    }

    /// Blank title -> `Err(InvalidTitle)`.
    pub fn rename(&self, new_title: &str) -> Result<(), MusicError> {
        todo!()
    }
}

/// A playlist groups songs but does not own them.
pub struct Playlist;

impl Playlist {
    pub fn new(name: &str) -> Playlist {
        todo!()
    }

    pub fn name(&self) -> &str {
        todo!()
    }

    /// Stores a clone of the `Rc` (shared, never a copy of the song). Returns false if that very `Rc` is already in.
    pub fn add(&mut self, song: Rc<Song>) -> bool {
        todo!()
    }

    /// Removes that very `Rc` (compare with `Rc::ptr_eq`). Returns whether it was present.
    pub fn remove(&mut self, song: &Rc<Song>) -> bool {
        todo!()
    }

    /// Songs in insertion order.
    pub fn songs(&self) -> &[Rc<Song>] {
        todo!()
    }

    pub fn total_seconds(&self) -> u32 {
        todo!()
    }

    /// Empties the playlist. The songs live on.
    pub fn clear(&mut self) {
        todo!()
    }
}
