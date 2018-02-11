#[macro_use]

extern crate log;
extern crate dylib;
extern crate xlib;
extern crate xinerama;

pub mod core {
    pub mod commands {
        pub use ::commands::{Cmd, CmdManage, ManageHook};
    }

    pub mod keycode {
        pub use ::keycode::*;
    }

    pub mod layout {
        pub use ::layout::*;
    }

    pub use ::config::{Config, Statusbar, LogInfo};
    pub use ::workspaces::WorkspaceConfig;
}

pub mod xlib_window_system;
pub mod config;
pub mod workspaces;
pub mod commands;
pub mod keycode;
pub mod layout;
