#![allow(dead_code)]

use std::default::Default;
use std::io::Write;
use std::process::{Command, Child, Stdio};
use layout::*;
use keycode::*;
use workspaces::{Workspaces, WorkspaceConfig};
use xlib_window_system::XlibWindowSystem;
use commands::{Cmd, ManageHook};
use dylib::DynamicLibrary;

pub struct Keybinding {
    pub mods: u8,
    pub key: String,
    pub cmd: Cmd,
}

pub struct WorkspaceInfo {
    pub id: usize,
    pub tag: String,
    pub screen: usize,
    pub current: bool,
    pub visible: bool,
    pub urgent: bool,
}

pub struct LogInfo {
    pub workspaces: Vec<WorkspaceInfo>,
    pub layout_name: String,
    pub window_title: String,
}

pub struct Statusbar {
    child: Option<Child>,
    executable: String,
    args: Option<Vec<String>>,
    fn_format: Box<Fn(LogInfo) -> String>,
}

impl Statusbar {
    pub fn new(executable: String,
               args: Option<Vec<String>>,
               fn_format: Box<Fn(LogInfo) -> String>)
               -> Statusbar {
        Statusbar {
            child: None,
            executable: executable,
            args: args,
            fn_format: fn_format,
        }
    }

    pub fn xmobar() -> Statusbar {
        let mut args = Vec::new();
        args.push("-f".to_string());
        args.push("xos4 Terminus 12".to_string());
        Statusbar::new("xmobar".to_string(),
                       Some(args),
                       Box::new(move |info: LogInfo| -> String {
            let workspaces = info.workspaces
                .iter()
                .map(|x| {
                    let (fg, bg) = if x.current {
                        ("#00ff00", "#000000")
                    } else if x.visible {
                        ("#009900", "#000000")
                    } else if x.urgent {
                        ("#ff0000", "#000000")
                    } else {
                        ("#ffffff", "#000000")
                    };
                    format!("<fc={},{}>[{}]</fc>", fg, bg, x.tag)
                })
                .collect::<Vec<String>>()
                .join(" ");

            format!("{} | {} | {}\n",
                    workspaces,
                    info.layout_name,
                    info.window_title)
        }))
    }

    pub fn start(&mut self) {
        match self.child {
            Some(_) => warn!("'{}' is already running", self.executable),
            None => {
                let mut cmd = Command::new(self.executable.clone());
                if self.args.is_some() {
                    cmd.args(self.args.clone().unwrap().as_slice());
                }
                match cmd.stdin(Stdio::piped()).spawn() {
                    Ok(child) => self.child = Some(child),
                    Err(err) => error!("failed to execute '{}': {}", self.executable, err),
                }
            }
        }
    }

    pub fn update(&mut self, ws: &XlibWindowSystem, workspaces: &Workspaces) {
        if self.child.is_none() {
            return;
        }

        let output = (self.fn_format)(LogInfo {
            workspaces: workspaces.all()
                .iter()
                .enumerate()
                .map(|(i, x)| {
                    WorkspaceInfo {
                        id: i,
                        tag: x.get_tag(),
                        screen: 0,
                        current: i == workspaces.get_index(),
                        visible: x.is_visible(),
                        urgent: x.is_urgent(),
                    }
                })
                .collect(),
            layout_name: workspaces.current().get_layout().name(),
            window_title: ws.get_window_title(workspaces.current().focused_window()),
        });

        match self.child.as_mut() {
            Some(child) => {
                match child.stdin.as_mut() {
                    Some(stdin) => {
                        stdin.write_all(output.as_bytes()).ok();
                    }
                    None => {
                        debug!{"Could not get stdin of child"};
                    }
                }
            }
            None => {
                debug!{"Could not get child"};
            }
        }
    }
}

pub struct Config {
    lib: Option<DynamicLibrary>,
    pub workspaces: Vec<WorkspaceConfig>,
    pub mod_key: u8,
    pub border_width: u32,
    pub border_color: u32,
    pub border_focus_color: u32,
    pub border_urgent_color: u32,
    pub greedy_view: bool,
    pub keybindings: Vec<Keybinding>,
    pub manage_hooks: Vec<ManageHook>,
    pub statusbar: Option<Statusbar>,
}

impl Default for Config {
    fn default() -> Config {
        let mut config = Config {
            lib: None,
            workspaces: (1usize..10)
                .map(|idx| {
                    WorkspaceConfig {
                        tag: idx.to_string(),
                        screen: 0,
                        layout: StrutLayout::new(TallLayout::new(1, 0.5, 0.05)),
                    }
                })
                .collect(),
            mod_key: MOD_4,
            border_width: 2,
            border_color: 0x002e2e2e,
            border_focus_color: 0x002a82e6,
            border_urgent_color: 0x00ff0000,
            greedy_view: false,
            keybindings: vec![Keybinding {
                                  mods: 0,
                                  key: "Return".to_string(),
                                  cmd: Cmd::Exec("alacritty".to_string()),
                              },
                              Keybinding {
                                  mods: 0,
                                  key: "p".to_string(),
                                  cmd: Cmd::Exec("rofi -show run".to_string()),
                              },
                              Keybinding {
                                  mods: MOD_SHIFT,
                                  key: "c".to_string(),
                                  cmd: Cmd::KillClient,
                              },
                              Keybinding {
                                  mods: 0,
                                  key: "j".to_string(),
                                  cmd: Cmd::FocusDown,
                              },
                              Keybinding {
                                  mods: 0,
                                  key: "k".to_string(),
                                  cmd: Cmd::FocusUp,
                              },
                              Keybinding {
                                  mods: 0,
                                  key: "m".to_string(),
                                  cmd: Cmd::FocusMaster,
                              },
                              Keybinding {
                                  mods: MOD_SHIFT,
                                  key: "j".to_string(),
                                  cmd: Cmd::SwapDown,
                              },
                              Keybinding {
                                  mods: MOD_SHIFT,
                                  key: "k".to_string(),
                                  cmd: Cmd::SwapUp,
                              },
                              Keybinding {
                                  mods: MOD_SHIFT,
                                  key: "Return".to_string(),
                                  cmd: Cmd::SwapMaster,
                              },
                              Keybinding {
                                  mods: 0,
                                  key: "comma".to_string(),
                                  cmd: Cmd::SendLayoutMsg(LayoutMsg::IncreaseMaster),
                              },
                              Keybinding {
                                  mods: 0,
                                  key: "period".to_string(),
                                  cmd: Cmd::SendLayoutMsg(LayoutMsg::DecreaseMaster),
                              },
                              Keybinding {
                                  mods: 0,
                                  key: "apostrophe".to_string(),
                                  cmd: Cmd::SendLayoutMsg(LayoutMsg::SplitVertical),
                              },
                              Keybinding {
                                  mods: MOD_SHIFT,
                                  key: "apostrophe".to_string(),
                                  cmd: Cmd::SendLayoutMsg(LayoutMsg::SplitHorizontal),
                              },
                              Keybinding {
                                  mods: 0,
                                  key: "l".to_string(),
                                  cmd: Cmd::SendLayoutMsg(LayoutMsg::Increase),
                              },
                              Keybinding {
                                  mods: 0,
                                  key: "h".to_string(),
                                  cmd: Cmd::SendLayoutMsg(LayoutMsg::Decrease),
                              },
                              Keybinding {
                                  mods: MOD_SHIFT,
                                  key: "q".to_string(),
                                  cmd: Cmd::Exit,
                              },
                              Keybinding {
                                  mods: MOD_SHIFT,
                                  key: "x".to_string(),
                                  cmd: Cmd::Reload,
                              }],
            manage_hooks: Vec::new(),
            statusbar: Some(Statusbar::xmobar()),
        };

        for i in 1..10 {
            config.keybindings.push(Keybinding {
                mods: 0,
                key: i.to_string(),
                cmd: Cmd::SwitchWorkspace(i),
            });

            config.keybindings.push(Keybinding {
                mods: MOD_SHIFT,
                key: i.to_string(),
                cmd: Cmd::MoveToWorkspace(i),
            });
        }

        for &(i, key) in vec![(1, "w"), (2, "e"), (3, "r")].iter() {
            config.keybindings.push(Keybinding {
                mods: 0,
                key: key.to_string(),
                cmd: Cmd::SwitchScreen(i),
            });

            config.keybindings.push(Keybinding {
                mods: MOD_SHIFT,
                key: key.to_string(),
                cmd: Cmd::MoveToScreen(i),
            });
        }

        config
    }
}

