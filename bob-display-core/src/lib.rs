//! Bob Display Core Library
//! 
//! Core rendering and display management functionality for the Bob Display system.
//! Provides DRM/KMS display access, configuration management, and bitmap font rendering.

pub mod config;
pub mod display;
pub mod render;

pub use config::Config;
pub use display::Display;
pub use render::Renderer;