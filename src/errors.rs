use std::io;
use xcb;

error_chain! {
    foreign_links {
        Xcb(xcb::ConnError);
        Io(io::Error);
    }
}
