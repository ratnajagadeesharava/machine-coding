//! Map markers and their icons.
#![allow(unused_variables)]

pub mod contract;

use contract::MapError;
use std::sync::Arc;

/// The icon of a map marker: shape, colour and a 4096-byte bitmap.
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

/// Provides `Arc<MarkerStyle>` values. Must be `Send + Sync`.
pub struct StyleFactory;

impl StyleFactory {
    pub fn new() -> StyleFactory {
        todo!()
    }

    /// Blank shape or colour -> `Err(MapError::InvalidStyle)`.
    pub fn get(&self, shape: &str, color: &str) -> Result<Arc<MarkerStyle>, MapError> {
        todo!()
    }

    /// How many `MarkerStyle` values this factory has created so far.
    pub fn instances_created(&self) -> usize {
        todo!()
    }
}

/// One pin on the map.
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

/// A layer of markers.
pub struct MapLayer;

impl MapLayer {
    pub fn new(factory: Arc<StyleFactory>) -> MapLayer {
        todo!()
    }

    /// Invalid arguments leave the layer unchanged.
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
