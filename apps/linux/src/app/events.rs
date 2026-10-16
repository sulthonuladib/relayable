use crate::api::{ApiCommand, ApiEvent, ConnectionState};
use crate::term::{TerminalModel, TerminalSize};

use super::state::{CreateIntent, CreateKind, Pane, SplitAxis, Tab};
use super::WorkspaceView;

impl WorkspaceView {
    // --- event application -------------------------------------------------

    pub(super) fn apply(&mut self, events: Vec<ApiEvent>) {
        for event in events {
            self.apply_one(event);
        }
    }

    fn apply_one(&mut self, event: ApiEvent) {
        match event {
            ApiEvent::ConnectionStateChanged(state) => {
                eprintln!("relayable-ui: connection state = {state:?}");
                self.connection = state.clone();
                if matches!(state, ConnectionState::Connected) {
                    self.api.send(ApiCommand::RefreshSessions);
                    // Stage-1 auto-session invariant (DEC-019): the `Connected`
                    // edge is one of the points where the app can be connected
                    // with zero tabs, so the check lives in `ensure_session`.
                    self.ensure_session();
                    // After a reconnect, existing panes have no live stream (they
                    // were invalidated on disconnect); attach them again so they
                    // re-seed from the returned snapshots.
                    self.reattach_missing_panes();
                }
            }
            ApiEvent::SessionsListed(sessions) => self.sessions = sessions,
            ApiEvent::SessionCreated {
                session_id,
                block_id,
                ..
            } => {
                eprintln!("relayable-ui: session.created session={session_id} block={block_id}");
                let intent = self.pending_create.take().unwrap_or(CreateIntent {
                    kind: CreateKind::NewTab,
                    label: None,
                });
                let pane_id = self.next_pane_id;
                self.next_pane_id += 1;
                let pane = Pane::new(pane_id, session_id.clone(), block_id, intent.label.clone());
                match intent.kind {
                    CreateKind::NewTab => {
                        self.tabs.push(Tab {
                            panes: vec![pane],
                            focus: 0,
                            axis: SplitAxis::Horizontal,
                        });
                        self.active_tab = self.tabs.len() - 1;
                    }
                    CreateKind::Split { axis } => {
                        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                            tab.axis = axis;
                            tab.panes.push(pane);
                            tab.focus = tab.panes.len() - 1;
                        }
                    }
                }
                // Match the server-side label by session id once the list lands.
                self.attach_pane(pane_id);
                self.api.send(ApiCommand::RefreshSessions);
            }
            ApiEvent::Attached {
                stream_id,
                cols,
                rows,
                screen_bytes,
            } => {
                eprintln!(
                    "relayable-ui: block.attached stream_id={stream_id} cols={cols} rows={rows} snapshot_bytes={}",
                    screen_bytes.len()
                );
                // Deterministic correlation: replies are processed in request
                // order, so the front of the queue is the pane that asked.
                let pane_id = self.pending_attach.pop_front();
                match pane_id {
                    Some(pane_id) => {
                        if let Some(pane) = self.pane_mut(pane_id) {
                            pane.stream_id = Some(stream_id);
                            pane.cols = cols;
                            pane.rows = rows;
                            pane.sent_cols = cols;
                            pane.sent_rows = rows;
                            pane.term = TerminalModel::new(TerminalSize::cells(cols, rows));
                            pane.term.process(&screen_bytes);
                            pane.history_offset = 0;
                            self.last_title_stream = Some(stream_id);
                        } else {
                            // The pane was closed while its attach was in
                            // flight. The FIFO slot was still consumed, so just
                            // release the stream it acquired.
                            eprintln!(
                                "relayable-ui: pane {pane_id} closed mid-attach; detaching stream_id={stream_id}"
                            );
                            self.api.send(ApiCommand::Detach { stream_id });
                        }
                    }
                    None => eprintln!(
                        "relayable-ui: Attached stream_id={stream_id} had no awaiting pane; ignoring"
                    ),
                }
            }
            ApiEvent::Output {
                stream_id, bytes, ..
            } => {
                self.last_title_stream = Some(stream_id);
                let total = self.stream_output_bytes.entry(stream_id).or_insert(0);
                *total += bytes.len() as u64;
                eprintln!(
                    "relayable-ui: output stream_id={stream_id} bytes={} total={total}",
                    bytes.len()
                );
                if let Some(pane) = self.pane_by_stream_mut(stream_id) {
                    pane.term.process(&bytes);
                }
            }
            ApiEvent::Resized {
                stream_id,
                cols,
                rows,
            } => {
                eprintln!(
                    "relayable-ui: terminal.resize accepted stream_id={stream_id} cols={cols} rows={rows}"
                );
                if let Some(pane) = self.pane_by_stream_mut(stream_id) {
                    pane.cols = cols;
                    pane.rows = rows;
                }
            }
            ApiEvent::PaneTitleChanged { program, cwd } => {
                eprintln!("relayable-ui: pane title/cwd program={program:?} cwd={cwd:?}");
                // `PaneTitleChanged` carries no stream id; the API emits it
                // immediately after the `Output`/`Attached` that produced it, so
                // attribute it to that stream.
                if let Some(stream_id) = self.last_title_stream {
                    if let Some(pane) = self.pane_by_stream_mut(stream_id) {
                        if program.is_some() {
                            pane.program = program;
                        }
                        if cwd.is_some() {
                            pane.cwd = cwd;
                        }
                    }
                }
            }
            ApiEvent::ServerEvent { kind, detail } => {
                // The API emits this when the data plane (re)connects. Re-attach
                // any pane whose stream was invalidated by a data-plane-only
                // disconnect, where the control plane stays connected and so no
                // `ConnectionStateChanged(Connected)` arrives.
                if kind == "data_plane_connected" {
                    // The terminal data path is usable again; clear a
                    // data-plane-only Disconnected banner so the UI does not stay
                    // stuck while panes work. The control plane re-emits its own
                    // state if it is still down.
                    if matches!(self.connection, ConnectionState::Disconnected) {
                        self.connection = ConnectionState::Connected;
                        eprintln!(
                            "relayable-ui: data plane reconnected; connection state restored to Connected"
                        );
                    }
                    // That restore is another path to `Connected` with zero tabs,
                    // so the auto-session invariant is re-checked here too.
                    self.ensure_session();
                    self.reattach_missing_panes();
                }
                self.push_status(format!("{kind}: {detail}"));
            }
            ApiEvent::Disconnected { reason } => {
                eprintln!("relayable-ui: disconnected reason={reason}");
                self.connection = ConnectionState::Disconnected;
                // Commands are not replayed across a restart of the control
                // plane (`api/control.rs` keeps draining the same queue), so a
                // `session.create` in flight when the transport dies is never
                // answered. Clear it, or the next `Connected` edge would find
                // `pending_create` still set and never create (DEC-019 defect 2).
                self.abandon_pending_create("connection lost");
                self.push_status(format!("disconnected: {reason}"));
                self.invalidate_streams();
                // The connection is gone: a queued attach reply will never
                // arrive, so drop the queue to avoid a stale FIFO head.
                self.pending_attach.clear();
            }
            ApiEvent::Error { message } => {
                eprintln!("relayable-ui: error {message}");
                self.connection = ConnectionState::Error(message.clone());
                // The only positive reply for a `session.create` is
                // `SessionCreated`, so an error while a create is in flight means
                // it did not succeed. Leaving the intent set would refuse every
                // later create (`create_session` returns early while it is set)
                // and wedge auto-create *and* `Ctrl+Shift+N` for the rest of the
                // run (DEC-019 defect 2).
                self.abandon_pending_create("error");
                // Re-check the invariant: the state is now `Error`, so
                // `needs_session` is false and this cannot re-send — a failed
                // create never spins into a retry loop. The retry is the next
                // `Connected` edge, or `Ctrl+Shift+N`.
                self.ensure_session();
                // A failed `block.attach` produced no `Attached`, so consume its
                // queue slot to keep the correlation FIFO aligned.
                if message.starts_with("block.attach") {
                    self.pending_attach.pop_front();
                }
                self.push_status(message);
                // Treat a request error like a disconnect: the stream the failed
                // request targeted may be dead, so suppress input/resize to it
                // until the next successful (re)attach.
                self.invalidate_streams();
            }
        }
    }

    pub(super) fn push_status(&mut self, message: String) {
        self.status.push(message);
        if self.status.len() > 64 {
            self.status.remove(0);
        }
    }
}
