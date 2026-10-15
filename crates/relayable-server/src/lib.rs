//! Relayable server library.
//!
//! The server owns sessions and their PTY children independently of any client
//! connection (`full.md` §1.3, §6.3). A dropped client never signals, reaps, or
//! stops a child; only `session.close` (or graceful server shutdown) does.
//!
//! The protocol front end, session registry, per-block PTY runtime, snapshot
//! projection, and signal handling live in sibling modules. This module exposes
//! only [`ServerConfig`] and [`Server`].

pub mod config;

mod connection;
mod dispatch;
mod registry;
mod session;
mod signals;
mod snapshot;

pub use config::ServerConfig;

use std::io::ErrorKind;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

use anyhow::Context;
use relayable_protocol::transport::bind_listener;

use crate::registry::ServerShared;

/// Lock a mutex, recovering from poisoning rather than panicking.
pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// A bound Relayable server.
pub struct Server {
    config: ServerConfig,
    shared: Arc<ServerShared>,
    listener: UnixListener,
}

impl Server {
    /// Bind the local Unix socket and prepare the registry.
    ///
    /// `bind_listener` creates the parent directory `0700`, removes only a stale
    /// socket, binds, and sets the socket file `0600`.
    ///
    /// # Errors
    ///
    /// Returns an error if the socket directory or socket cannot be created, or
    /// if a live listener already owns the path.
    pub fn bind(config: ServerConfig) -> anyhow::Result<Server> {
        let listener = bind_listener(&config.socket_path)
            .with_context(|| format!("failed to bind {}", config.socket_path.display()))?;
        let shared = Arc::new(ServerShared::new(config.clone()));
        Ok(Server {
            config,
            shared,
            listener,
        })
    }

    /// The bound socket path.
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.config.socket_path
    }

    /// Request graceful shutdown and wake every active connection. Idempotent.
    pub fn shutdown(&self) {
        self.shared.request_shutdown();
        for connection in lock(&self.shared.connections).iter() {
            connection.close_now();
        }
    }

    /// Run the accept loop until a shutdown signal is observed, then terminate
    /// every live child, join connection threads, and unlink the socket.
    ///
    /// # Errors
    ///
    /// Returns an error if the listener cannot be made non-blocking.
    pub fn run(self) -> anyhow::Result<()> {
        signals::install();
        let Server {
            config,
            shared,
            listener,
        } = self;
        listener.set_nonblocking(true)?;

        while !shared.shutdown.load(Ordering::SeqCst) && !signals::shutdown_requested() {
            match listener.accept() {
                Ok((stream, _)) => {
                    let _ = stream.set_nonblocking(false);
                    let handle = connection::spawn_connection(Arc::clone(&shared), stream);
                    let mut threads = lock(&shared.connection_threads);
                    // Drop handles for connections that have already finished so
                    // the list tracks live connections, not all historical ones.
                    threads.retain(|handle| !handle.is_finished());
                    threads.push(handle);
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    lock(&shared.connection_threads).retain(|handle| !handle.is_finished());
                    thread::sleep(Duration::from_millis(20));
                }
                Err(error) => {
                    eprintln!("relayable-server: accept failed: {error}");
                    thread::sleep(Duration::from_millis(20));
                }
            }
        }

        shared.request_shutdown();
        for connection in lock(&shared.connections).iter() {
            connection.close_now();
        }
        for session in shared.live_sessions() {
            session.block.close(Duration::from_millis(500));
        }
        let threads: Vec<_> = std::mem::take(&mut *lock(&shared.connection_threads));
        for handle in threads {
            let _ = handle.join();
        }
        let _ = std::fs::remove_file(&config.socket_path);
        Ok(())
    }
}
