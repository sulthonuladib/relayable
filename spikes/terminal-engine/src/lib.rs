//! Shared helpers for the Relayable terminal-engine spike (TERM-001 / ADR-003).
//!
//! This crate is engine-independent ground truth: it owns a real PTY, drives a
//! real child process, and records raw bytes / kernel state. It deliberately
//! contains no terminal-emulation engine and no renderer.

pub mod esc;
pub mod pty;
