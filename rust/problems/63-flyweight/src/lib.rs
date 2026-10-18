//! Flyweight — markers share their heavy icon (`Arc<MarkerStyle>`) and keep only a position and a label.
#![allow(unused_variables)]

pub mod contract;

use contract::MapError;
use std::sync::Arc;

/// The flyweight: shape, colour and a 4096-byte bitmap allocated once per style. Add a private constructor that
/// only `StyleFactory` uses: nobody else should be able to create one.
pub struct MarkerStyle;

impl MarkerStyle {
    pub fn shape(&self) -> &str {
        todo!()
    }

    pub fn color(&self) -> &str {
        todo!()
    }

    /// Size of the icon bitmap in bytes (always 4096).
    pub fn bitmap_size(&self) -> usize {
        todo!()
    }
}

/// Hands out ONE shared `Arc<MarkerStyle>` per distinct (shape, colour). Thread-safe (`Send + Sync`).
pub struct StyleFactory;

impl StyleFactory {
    pub fn new() -> StyleFactory {
        todo!()
    }

    /// The shared style for this shape and colour (compared exactly, case-sensitive). Blank shape or colour ->
    /// `Err(MapError::InvalidStyle)`. Many threads may ask at once; each distinct pair is still created only once.
    pub fn get(&self, shape: &str, color: &str) -> Result<Arc<MarkerStyle>, MapError> {
        todo!()
    }

    /// How many `MarkerStyle` values this factory has created so far.
    pub fn instances_created(&self) -> usize {
        todo!()
    }
}

/// One pin: EXTRINSIC (per-pin) state only - position and label - plus the shared style. It must never hold a
/// bitmap of its own.
pub struct Marker;

impl Marker {
    /// lat -90..=90, lng -180..=180, otherwise `Err(MapError::InvalidPosition)`.
    pub fn new(lat: f64, lng: f64, label: &str, style: Arc<MarkerStyle>) -> Result<Marker, MapError> {
        todo!()
    }

    pub fn lat(&self) -> f64 {
        todo!()
    }

    pub fn lng(&self) -> f64 {
        todo!()
    }

    pub fn label(&self) -> &str {
        todo!()
    }

    pub fn style(&self) -> &Arc<MarkerStyle> {
        todo!()
    }
}

/// A layer of markers. Styles always come from the factory it was given.
pub struct MapLayer;

impl MapLayer {
    pub fn new(factory: Arc<StyleFactory>) -> MapLayer {
        todo!()
    }

    /// Creates a marker whose style is `factory.get(shape, color)`. Invalid arguments leave the layer unchanged.
    pub fn add_marker(&mut self, lat: f64, lng: f64, label: &str, shape: &str, color: &str) -> Result<(), MapError> {
        todo!()
    }

    pub fn marker_count(&self) -> usize {
        todo!()
    }

    pub fn markers(&self) -> &[Marker] {
        todo!()
    }

    /// One line per marker, in order: `"<shape>/<color> @ <lat>,<lng> <label>"`, e.g. `"pin/red @ 12.5,-3.25 Cafe"`.
    pub fn render(&self) -> Vec<String> {
        todo!()
    }
}
