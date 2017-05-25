#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate slog;
#[macro_use]
extern crate slog_scope;
extern crate slog_term;
extern crate slog_stream;
extern crate slog_json;
extern crate xcb;

mod errors;
mod window_system;

use errors::*;
use window_system::{WindowSystem, XWindowSystem};

use std::fs::OpenOptions;

fn init_logger() -> Result<()> {
    use slog::DrainExt;

    let log_path = concat!(env!("HOME"), "/.xr3wm/log");
    let log_file = OpenOptions::new().create(true).write(true).truncate(true).open(log_path).chain_err(|| "unable to create log file")?;

    let term_drain = slog_term::streamer().compact().build();
    let file_drain = slog_stream::stream(log_file, slog_json::default());
    slog_scope::set_global_logger(slog::Logger::root(slog::duplicate(term_drain, file_drain).fuse(), o![]));

    Ok(())
}

fn run() -> Result<()> {
    init_logger()?;

    info!("initializing logger");

    let backend = XWindowSystem::initialize().chain_err(|| "unable to initialize Window System")?;

    backend.run()
}

fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);
        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        std::process::exit(1);
    }
}
