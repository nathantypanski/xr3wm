#![allow(dead_code, unused_must_use)]

extern crate libc;

use self::libc::execvp;
use std::thread;
use std::ptr::null;
use std::env;
use std::ffi::CString;
use std::process::Command;
use std::io::prelude::*;
use std::path::Path;
use std::fs::{OpenOptions, remove_file};
use config::Config;
use layout::LayoutMsg;
use xlib_window_system::XlibWindowSystem;
use workspaces::{Workspaces, MoveOp};
use xlib::Window;

pub enum Cmd {
    Exec(String),
    SwitchWorkspace(usize),
    SwitchScreen(usize),
    MoveToWorkspace(usize),
    MoveToScreen(usize),
    SendLayoutMsg(LayoutMsg),
    Reload,
    Exit,
    KillClient,
    FocusUp,
    FocusDown,
    FocusMaster,
    SwapUp,
    SwapDown,
    SwapMaster,
}

impl Cmd {
    pub fn call(&self, ws: &XlibWindowSystem, workspaces: &mut Workspaces, config: &Config) {
        match *self {
            Cmd::Exec(ref cmd) => {
                debug!("Cmd::Exec: {}", cmd);
                exec(cmd.clone());
            }
            Cmd::SwitchWorkspace(index) => {
                debug!("Cmd::SwitchWorkspace: {}", index);
                workspaces.switch_to(ws, config, index - 1);
            }
            Cmd::SwitchScreen(screen) => {
                debug!("Cmd::SwitchScreen: {}", screen);
                workspaces.switch_to_screen(ws, config, screen - 1);
            }
            Cmd::MoveToWorkspace(index) => {
                debug!("Cmd::MoveToWorkspace: {}", index);
                workspaces.move_window_to(ws, config, index - 1);
            }
            Cmd::MoveToScreen(screen) => {
                debug!("Cmd::MoveToScreen: {}", screen);
                workspaces.move_window_to_screen(ws, config, screen - 1);
            }
            Cmd::SendLayoutMsg(ref msg) => {
                debug!("Cmd::SendLayoutMsg::{:?}", msg);
                workspaces.current_mut().map(|ws| ws.send_layout_message(msg.clone()));
                workspaces.current().redraw(ws, config);
            }
            Cmd::Reload => {
                let curr_exe = env::current_exe().unwrap();
                let filename = curr_exe.file_name().unwrap().to_str().unwrap();

                println!("recompiling...");
                debug!("Cmd::Reload: compiling...");

                let mut cmd = Command::new("cargo");
                cmd.current_dir(&env::current_dir().unwrap()).arg("build").env("RUST_LOG", "none");

                match cmd.output() {
                    Ok(output) => {
                        if output.status.success() {
                            debug!("Cmd::Reload: restarting xr3wm...");

                            unsafe {
                                let mut slice : &mut [*const i8; 2] = &mut [
                  CString::new(filename.as_bytes()).unwrap().as_ptr() as *const i8,
                  null()
                ];

                                let path = Path::new(concat!(env!("HOME"), "/.xr3wm/.tmp"));
                                if path.exists() {
                                    remove_file(&path);
                                }

                                let mut file = OpenOptions::new().write(true).open(&path).unwrap();
                                file.write_all(workspaces.serialize().as_bytes());
                                file.flush();

                                execvp(CString::new(curr_exe.to_str().unwrap().as_bytes()).unwrap().as_ptr() as *const i8, slice.as_mut_ptr());
                            }
                        } else {
                            panic!("failed to recompile: '{}'", output.status);
                        }
                    }
                    _ => panic!("failed to start \"{:?}\"", cmd),
                }
            }
            Cmd::Exit => {
                debug!("Cmd::Exit");
                ws.close();
            }
            Cmd::KillClient => {
                match workspaces.current_mut() {
                    Some(wss) => {
                        debug!("Cmd::KillClient: {:?}", wss.focused_window());
                        ws.kill_window(wss.focused_window());
                    }
                    None => {}
                }
            }
            Cmd::FocusUp => {
                debug!("Cmd::FocusUp: {}", workspaces.current().focused_window());
                match workspaces.current_mut() {
                    Some(wss) => wss.move_focus(ws, config, MoveOp::Up),
                    None => {}
                }
            }
            Cmd::FocusDown => {
                debug!("Cmd::FocusDown: {}", workspaces.current().focused_window());
                match workspaces.current_mut() {
                    Some(wss) => wss.move_focus(ws, config, MoveOp::Down),
                    None => {}
                }
            }
            Cmd::FocusMaster => {
                debug!("Cmd::FocusMaster: {}",
                       workspaces.current().focused_window());
                match workspaces.current_mut() {
                    Some(wss) => wss.move_focus(ws, config, MoveOp::Swap),
                    None => debug!("Cmd::FocusMaster on no workspace"),
                }
            }
            Cmd::SwapUp => {
                debug!("Cmd::SwapUp: {}", workspaces.current().focused_window());
                match workspaces.current_mut() {
                    Some(wss) => wss.move_window(ws, config, MoveOp::Up),
                    None => debug!("Cmd::SwapDown on no workspace")
                }
            }
            Cmd::SwapDown => {
                debug!("Cmd::SwapDown: {}", workspaces.current().focused_window());
                match workspaces.current_mut() {
                    Some(wss) => wss.move_window(ws, config, MoveOp::Down),
                    None => debug!("Cmd::SwapDown on no workspace")
                }
            }
            Cmd::SwapMaster => {
                debug!("Cmd::SwapMaster: {}", workspaces.current().focused_window());
                match workspaces.current_mut() {
                    Some(wss) => wss.move_window(ws, config, MoveOp::Swap),
                    None => {}
                }
            }
        }
    }
}

pub struct ManageHook {
    pub class_name: String,
    pub cmd: CmdManage,
}

pub enum CmdManage {
    Move(usize),
    Float,
    Fullscreen,
    Ignore,
}

impl CmdManage {
    pub fn call(&self,
                ws: &XlibWindowSystem,
                workspaces: &mut Workspaces,
                config: &Config,
                window: Window) {
        match *self {
            CmdManage::Move(index) => {
                if let Some(parent) = ws.transient_for(window) {
                    if let Some(workspace) = workspaces.find_window(parent) {
                        workspace.add_window(ws, config, window);
                        workspace.focus_window(ws, config, window);
                    }
                } else {
                    debug!("CmdManage::Move: {}, {}", window, index);
                    match workspaces.get_mut(index - 1) {
                        Some(wss) => wss.add_window(ws, config, window),
                        None => {}
                    }
                    match workspaces.get_mut(index - 1) {
                        Some(wss) => wss.focus_window(ws, config, window),
                        None => {}
                    }
                }
            }
            CmdManage::Float => {
                debug!("CmdManage::Float");
                unimplemented!()
            }
            CmdManage::Fullscreen => {
                debug!("CmdManage::Fullscreen");
                unimplemented!()
            }
            CmdManage::Ignore => {
                debug!("CmdManage::Ignore");
                unimplemented!()
            }
        }
    }
}

fn exec(cmd: String) {
    thread::spawn(move || {
        let args: Vec<&str> = cmd[..].split(' ').collect();

        if args.len() > 0 {
            let mut cmd = Command::new(args[0]);

            if args.len() > 1 {
                cmd.args(&args[1..]);
            }

            match cmd.output() {
                Ok(_) => (),
                _ => panic!("failed to start \"{:?}\"", cmd),
            }
        }
    });
}
