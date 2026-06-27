//! kime-ibus: an IBus engine that forwards key events to the kime input engine.
//!
//! This is the path for GNOME Wayland (Mutter lacks `zwp_input_method_v2`), where
//! IBus is the supported input-method protocol. See issues #422 and #748.

mod connection;
mod engine;
mod factory;
mod ibus_types;

pub use connection::run;
