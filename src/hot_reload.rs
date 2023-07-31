use engine::Engine;
use sdl2::keyboard::Keycode;
use std::process::Child;
use std::process::Command;

use crate::game;
use crate::WINDOW_TITLE;

const ANIMATION_PERIOD: u128 = 500;

#[derive(PartialEq)]
enum CommandStatus {
    Idle,
    Running,
    Done,
    Failed,
}

pub struct HotReloader {
    build_cmd: Command,
    build_cmd_invocation: Option<Child>,
    animation_time_ms: u128,
}

impl HotReloader {
    pub fn new() -> Self {
        let mut build_cmd = std::process::Command::new("cargo");
        build_cmd.args(["build", "-p", "game"]);
        HotReloader {
            build_cmd,
            build_cmd_invocation: None,
            animation_time_ms: 0,
        }
    }

    pub fn update(&mut self, engine: &mut Engine) {
        if cfg!(debug_assertions) {
            if game::was_reloaded() {
                on_game_reloaded();
            }

            match self.command_status() {
                CommandStatus::Idle => {
                    if engine.input.keyboard.is_pressed_now(Keycode::F5) {
                        self.rebuild_game_lib();
                    }
                }
                CommandStatus::Running => {
                    self.animate_window_title(engine);
                }
                CommandStatus::Done => {
                    let _ = engine.window.set_title(WINDOW_TITLE);
                    self.build_cmd_invocation = None;
                }
                CommandStatus::Failed => {
                    let _ = engine
                        .window
                        .set_title(&format!("{} (!!! build errors !!!)", WINDOW_TITLE));
                    self.build_cmd_invocation = None;
                }
            }
        }
    }

    fn animate_window_title(&mut self, engine: &mut Engine) {
        self.animation_time_ms += engine.frame.delta_ms;
        self.animation_time_ms %= 4 * ANIMATION_PERIOD;

        let title_suffix = if self.animation_time_ms < ANIMATION_PERIOD {
            "(building...)"
        } else if self.animation_time_ms < 2 * ANIMATION_PERIOD {
            "(building   )"
        } else if self.animation_time_ms < 3 * ANIMATION_PERIOD {
            "(building.  )"
        } else {
            "(building.. )"
        };

        let _ = engine
            .window
            .set_title(&format!("{} {}", WINDOW_TITLE, title_suffix));
    }

    fn rebuild_game_lib(&mut self) {
        log::info!("Rebuilding game code");
        self.animation_time_ms = 0;
        self.build_cmd_invocation =
            Some(self.build_cmd.spawn().expect("failed to execute process"));
    }

    fn command_status(&mut self) -> CommandStatus {
        if let Some(invocation) = &mut self.build_cmd_invocation {
            if let Some(status) = invocation.try_wait().unwrap() {
                if status.success() {
                    CommandStatus::Done
                } else {
                    CommandStatus::Failed
                }
            } else {
                CommandStatus::Running
            }
        } else {
            CommandStatus::Idle
        }
    }
}

fn on_game_reloaded() {
    unsafe {
        log::info!("Game code hot-reloaded");
        game::set_global_contexts(
            log::logger(),
            log::max_level(),
            imgui::sys::igGetCurrentContext(),
        );
    }
}
