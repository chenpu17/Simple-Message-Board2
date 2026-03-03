//! 简易留言板 - Rust 后端
//!
//! 提供留言板的核心功能和 API

pub mod cli;
pub mod config;
pub mod daemon;
pub mod db;
pub mod handlers;
pub mod utils;

pub use db::Repository;
