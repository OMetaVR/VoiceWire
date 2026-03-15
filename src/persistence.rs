use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::model::SessionState;

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedSession {
    strips: Vec<PersistedStrip>,
    buses: Vec<PersistedBus>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedStrip {
    device_name: String,
    gain_db: f32,
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
    std::thread::spawn(move || {
        save_session_inner(persisted);
    });
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
    let _ = fs::write(path, json);
}

fn apply_persisted_session(session: &mut SessionState, persisted: PersistedSession) {
    for (strip, saved) in session.strips.iter_mut().zip(persisted.strips) {
        strip.device_name = saved.device_name;
        strip.gain_db = saved.gain_db.clamp(-60.0, 12.0);
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
        }
    }
}
