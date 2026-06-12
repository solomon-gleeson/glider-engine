use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver};

use bevy::prelude::*;

use crate::instance::{ConsoleLevel, ScriptConsole};

pub(super) enum BuildMsg {
    Done { ok: bool, summary: String },
}

#[derive(Resource, Default)]
pub struct BuildState {
    pub running: bool,
    pub rx: Option<Mutex<Receiver<BuildMsg>>>,
}

pub fn drive_build(
    mut build: ResMut<BuildState>,
    mut requested: ResMut<super::editor_state::EditorState>,
    mut console: ResMut<ScriptConsole>,
) {
    if requested.build_requested && !build.running {
        requested.build_requested = false;
        build.running = true;
        console.push(
            ConsoleLevel::Info,
            "Building game (render only)\u{2026}".into(),
        );

        let (tx, rx) = mpsc::channel();
        build.rx = Some(Mutex::new(rx));
        std::thread::spawn(move || {
            let output = std::process::Command::new("cargo")
                .args(["build", "--no-default-features", "--features", "render"])
                .output();
            let msg = match output {
                Ok(out) => {
                    let ok = out.status.success();
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let summary = stderr.lines().rev().take(8).collect::<Vec<_>>();
                    BuildMsg::Done {
                        ok,
                        summary: summary.into_iter().rev().collect::<Vec<_>>().join("\n"),
                    }
                }
                Err(e) => BuildMsg::Done {
                    ok: false,
                    summary: format!("failed to launch cargo: {e}"),
                },
            };
            let _ = tx.send(msg);
        });
    }

    let received = build
        .rx
        .as_ref()
        .and_then(|m| m.lock().ok()?.try_recv().ok());
    if build.running
        && let Some(BuildMsg::Done { ok, summary }) = received
    {
        let level = if ok {
            ConsoleLevel::Info
        } else {
            ConsoleLevel::Error
        };
        console.push(
            level,
            if ok {
                "Build succeeded \u{2014} target/debug/glider".into()
            } else {
                "Build failed:".into()
            },
        );
        if !ok {
            for line in summary.lines() {
                console.push(ConsoleLevel::Error, line.to_string());
            }
        }
        build.running = false;
        build.rx = None;
    }
}
