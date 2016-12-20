#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

//#[macro_use(o, slog_log, slog_trace, slog_debug, slog_info, slog_warn, slog_error)]
#[macro_use]
extern crate slog;
#[macro_use]
extern crate slog_scope;
extern crate slog_term;
extern crate slog_stream;
extern crate slog_json;

extern crate xcb;

mod errors;

use errors::*;
use slog::DrainExt;

use std::fs::OpenOptions;

fn init_logger() -> Result<()> {
    let log_path = concat!(env!("HOME"), "/.xr3wm/log");
    let log_file = OpenOptions::new().create(true).write(true).truncate(true).open(log_path).chain_err(|| "unable to create log file")?;

    let term_drain = slog_term::streamer().compact().build();
    let file_drain = slog_stream::stream(log_file, slog_json::default());
    slog_scope::set_global_logger(slog::Logger::root(slog::duplicate(term_drain, file_drain).fuse(), o![]));

    Ok(())
}

fn run() -> Result<()> {
    init_logger()?;

    info!("initialized logger");

    Ok(())
}

fn main() {
    if let Err(ref e) = run() {
        error!("{}", e);
        for e in e.iter().skip(1) {
            error!("caused by: {}", e);
        }
        if let Some(backtrace) = e.backtrace() {
            error!("backtrace: {:?}", backtrace);
        }
        std::process::exit(1);
    }
}
