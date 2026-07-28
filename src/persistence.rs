use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::model::SessionState;

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedSession {
    strips: Vec<PersistedStrip>,
    buses: Vec<PersistedBus>,
    #[serde(default, alias = "appRoutes", alias = "app_routes")]
    remembered_app_routes: Vec<PersistedAppRoute>,
    #[serde(default)]
    quick_route_enabled: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedStrip {
    device_name: String,
    gain_db: f32,
    #[serde(default)]
    gate: f32,
    muted: bool,
    solo: bool,
    mono: bool,
    routes: Vec<bool>,
    apps: Vec<PersistedApp>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedApp {
    level: f32,
    muted: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedBus {
    destination_name: String,
    gain_db: f32,
    muted: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedAppRoute {
    name: String,
    detail: String,
    target: String,
    icon_text: String,
    icon_color: String,
    level: f32,
    muted: bool,
}

struct SaveQueue {
    pending: Mutex<Option<PersistedSession>>,
    writer: Mutex<()>,
    wake: Condvar,
}

static SAVE_QUEUE: OnceLock<Arc<SaveQueue>> = OnceLock::new();

pub fn load_session() -> Option<SessionState> {
    let path = session_path()?;
    let contents = fs::read_to_string(path).ok()?;
    let persisted: PersistedSession = serde_json::from_str(&contents).ok()?;
    let mut session = SessionState::demo();
    apply_persisted_session(&mut session, persisted);
    Some(session)
}

pub fn save_session_async(session: SessionState) {
    let persisted = PersistedSession::from(&session);
    let queue = SAVE_QUEUE.get_or_init(|| {
        let queue = Arc::new(SaveQueue {
            pending: Mutex::new(None),
            writer: Mutex::new(()),
            wake: Condvar::new(),
        });
        let worker_queue = Arc::clone(&queue);
        std::thread::spawn(move || {
            loop {
                let mut pending = worker_queue
                    .pending
                    .lock()
                    .expect("session save queue mutex should not be poisoned");
                while pending.is_none() {
                    pending = worker_queue
                        .wake
                        .wait(pending)
                        .expect("session save queue mutex should not be poisoned");
                }
                drop(pending);

                std::thread::sleep(Duration::from_millis(120));

            let _writer = worker_queue
                .writer
                .lock()
                .expect("session writer mutex should not be poisoned");
            let persisted = worker_queue
                .pending
                .lock()
                    .expect("session save queue mutex should not be poisoned")
                    .take();
                if let Some(persisted) = persisted {
                    save_session_inner(persisted);
                }
            }
        });
        queue
    });
    *queue
        .pending
        .lock()
        .expect("session save queue mutex should not be poisoned") = Some(persisted);
    queue.wake.notify_one();
}

pub fn flush_session_saves() {
    let Some(queue) = SAVE_QUEUE.get() else {
        return;
    };
    let _writer = queue
        .writer
        .lock()
        .expect("session writer mutex should not be poisoned");
    let persisted = queue
        .pending
        .lock()
        .expect("session save queue mutex should not be poisoned")
        .take();
    if let Some(persisted) = persisted {
        save_session_inner(persisted);
    }
}

fn save_session_inner(persisted: PersistedSession) {
    let Some(path) = session_path() else {
        return;
    };
    let Some(parent) = path.parent() else {
        return;
    };
    if fs::create_dir_all(parent).is_err() {
        return;
    }

    let Ok(json) = serde_json::to_string_pretty(&persisted) else {
        return;
    };
    let temporary_path = path.with_extension("json.tmp");
    if fs::write(&temporary_path, json).is_ok() {
        let _ = fs::rename(temporary_path, path);
    }
}

fn apply_persisted_session(session: &mut SessionState, persisted: PersistedSession) {
    session.set_quick_route_enabled(persisted.quick_route_enabled);

    for (strip, saved) in session.strips.iter_mut().zip(persisted.strips) {
        strip.device_name = saved.device_name;
        strip.gain_db = saved.gain_db.clamp(-60.0, 12.0);
        strip.gate = saved.gate.clamp(0.0, 10.0);
        strip.muted = saved.muted;
        strip.solo = saved.solo;
        strip.mono = saved.mono;

        for (route, enabled) in strip.routes.iter_mut().zip(saved.routes) {
            route.enabled = enabled;
        }

        for (app, saved_app) in strip.apps.iter_mut().zip(saved.apps) {
            app.level = saved_app.level.clamp(0.0, 1.0);
            app.muted = saved_app.muted;
        }
    }

    for (bus, saved) in session.buses.iter_mut().zip(persisted.buses) {
        bus.destination_name = saved.destination_name;
        bus.gain_db = saved.gain_db.clamp(-60.0, 12.0);
        bus.muted = saved.muted;
    }

    if !persisted.remembered_app_routes.is_empty() {
        session.set_remembered_app_routes(
            persisted
                .remembered_app_routes
                .into_iter()
                .map(|route| crate::model::RememberedAppRouteState {
                    name: route.name,
                    detail: route.detail,
                    target: route.target,
                    icon_text: route.icon_text,
                    icon_color: route.icon_color,
                    level: route.level.clamp(0.0, 1.0),
                    muted: route.muted,
                })
                .collect(),
        );
    }
}

fn session_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| {
                let mut path = PathBuf::from(home);
                path.push(".config");
                path
            })
        })?;

    Some(base.join("voicewire").join("session.json"))
}

impl From<&SessionState> for PersistedSession {
    fn from(session: &SessionState) -> Self {
        Self {
            strips: session
                .strips
                .iter()
                .map(|strip| PersistedStrip {
                    device_name: strip.device_name.clone(),
                    gain_db: strip.gain_db,
                    gate: strip.gate,
                    muted: strip.muted,
                    solo: strip.solo,
                    mono: strip.mono,
                    routes: strip.routes.iter().map(|route| route.enabled).collect(),
                    apps: strip
                        .apps
                        .iter()
                        .map(|app| PersistedApp {
                            level: app.level,
                            muted: app.muted,
                        })
                        .collect(),
                })
                .collect(),
            buses: session
                .buses
                .iter()
                .map(|bus| PersistedBus {
                    destination_name: bus.destination_name.clone(),
                    gain_db: bus.gain_db,
                    muted: bus.muted,
                })
                .collect(),
            remembered_app_routes: session
                .remembered_app_routes
                .iter()
                .map(|route| PersistedAppRoute {
                    name: route.name.clone(),
                    detail: route.detail.clone(),
                    target: route.target.clone(),
                    icon_text: route.icon_text.clone(),
                    icon_color: route.icon_color.clone(),
                    level: route.level,
                    muted: route.muted,
                })
                .collect(),
            quick_route_enabled: session.quick_route_enabled,
        }
    }
}
