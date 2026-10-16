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
mod events;
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
use relayable_store::{PruneLimits, Store, StoreError};

use crate::events::DisconnectReason;
use crate::registry::{Persistence, ServerShared};

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
    /// Persistence is opened first (see [`open_store`]): when it fails because a
    /// peer already owns the state directory, startup is refused; any other
    /// store error degrades to in-memory-only operation with a warning.
    ///
    /// # Errors
    ///
    /// Returns an error if the socket directory or socket cannot be created, if
    /// a live listener already owns the path, or if another server already owns
    /// the state directory.
    pub fn bind(config: ServerConfig) -> anyhow::Result<Server> {
        let persistence = open_store(&config)?;
        let listener = bind_listener(&config.socket_path)
            .with_context(|| format!("failed to bind {}", config.socket_path.display()))?;
        let shared = Arc::new(ServerShared::new(config.clone(), persistence));
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
            connection.close_now(DisconnectReason::ServerShutdown);
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
            connection.close_now(DisconnectReason::ServerShutdown);
        }
        for session in shared.live_sessions() {
            session.block.close(Duration::from_millis(500));
        }
        let threads: Vec<_> = std::mem::take(&mut *lock(&shared.connection_threads));
        for handle in threads {
            let _ = handle.join();
        }
        // Persistence shutdown ordering (ADR-004 D2): after every session is
        // closed and every connection thread is joined, best-effort checkpoint
        // the WAL and then drop the store, which closes the database cleanly and
        // releases the single-writer lock before the process exits. The socket
        // is unlinked last, preserving the existing shutdown ordering.
        if let Some(persistence) = shared.take_persistence() {
            if let Err(error) = persistence.store.checkpoint() {
                eprintln!("relayable-server: persistence shutdown checkpoint failed: {error}");
            }
            drop(persistence);
        }
        let _ = std::fs::remove_file(&config.socket_path);
        Ok(())
    }
}

/// Open the durable store for this run, applying startup reconciliation and
/// pruning when it opens.
///
/// Returns `Ok(None)` when persistence is unavailable for a legitimate reason
/// (no resolvable state directory, or a non-lock open/migrate/reconcile error)
/// after logging a warning; the server then runs with in-memory metadata only.
/// The one error that refuses startup is [`StoreError::StateDirLocked`]: a
/// second server must never become a second writer (ADR-004 D2.5).
///
/// The retention limits are resolved **once** here from the `RELAYABLE_STORE_*`
/// environment ([`PruneLimits::from_env`]) and kept with the store, so every
/// server-side prune (startup and close) honors the documented overrides. Only
/// the age anchor is refreshed at each prune
/// ([`PruneLimits::with_current_time`]) so rows written after startup can still
/// age out; the environment is never re-read after startup.
fn open_store(config: &ServerConfig) -> anyhow::Result<Option<Persistence>> {
    let Some(state_dir) = config.state_dir.as_deref() else {
        eprintln!(
            "relayable-server: persistence disabled: no state directory resolved \
             (set RELAYABLE_STATE_DIR, or XDG_STATE_HOME, or HOME); running with \
             in-memory metadata only"
        );
        return Ok(None);
    };

    let store = match Store::open(state_dir) {
        Ok(store) => store,
        Err(StoreError::StateDirLocked(path)) => {
            anyhow::bail!(
                "another relayable-server already owns the state directory {}; refusing to \
                 start as a second writer",
                path.display()
            );
        }
        Err(error) => {
            eprintln!(
                "relayable-server: persistence disabled: failed to open {} ({error}); running \
                 with in-memory metadata only",
                state_dir.display()
            );
            return Ok(None);
        }
    };

    // A row left `active` belongs to a process that is gone: mark it
    // `interrupted` so it is never reported live (ADR-004 D2.5).
    match store.reconcile_on_startup() {
        Ok(report) if report.interrupted > 0 => {
            eprintln!(
                "relayable-server: reconciled {} stale session(s) to interrupted",
                report.interrupted
            );
        }
        Ok(_) => {}
        Err(error) => {
            eprintln!(
                "relayable-server: persistence disabled: startup reconciliation failed ({error}); \
                 running with in-memory metadata only"
            );
            return Ok(None);
        }
    }

    // Retention limits are resolved once for the whole run (F1); the age anchor
    // is refreshed at every prune so post-startup rows can age out (F7).
    let limits = PruneLimits::from_env();

    // Pruning runs on startup and on close (ADR-004 D2.4); a prune failure is
    // not fatal because the store itself opened and reconciled.
    if let Err(error) = store.prune(&limits.clone().with_current_time()) {
        eprintln!("relayable-server: persistence startup prune failed: {error}");
    }

    Ok(Some(Persistence {
        store: Arc::new(store),
        limits,
    }))
}
