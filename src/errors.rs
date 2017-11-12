use std;
use xcb;
use log;

error_chain! {
    foreign_links {
        Xcb(xcb::ConnError);
        Fern(log::SetLoggerError);
        Io(std::io::Error);
    }
}
