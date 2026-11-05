//! Zacess Profile: one accessibility profile for every app.
//!
//! A profile states what a person needs (contrast floors, font, motion, cursor and spacing) and may
//! carry a palette. The library derives any colour the palette leaves out, checks every pair
//! against the floors and renders the result for terminals and editors.

pub mod check;
pub mod colour;
pub mod derive;
pub mod osc;
pub mod profile;
pub mod render;

pub use check::{Report, check};
pub use derive::{Derived, derive};
pub use profile::Profile;
