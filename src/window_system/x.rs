use errors::*;

use slog_scope;
use slog::Logger;
use xcb;
use window_system::*;

pub struct XWindowSystem {
    conn: xcb::Connection,
    screens: i32,
    log: Logger
}

impl WindowSystem for XWindowSystem {
    fn initialize() -> Result<XWindowSystem> {
        let (conn, screens) = xcb::Connection::connect(None)?;
        conn.has_error()?;

        Ok(XWindowSystem {
            conn: conn,
            screens: screens,
            log: slog_scope::logger().new(o!("xcb" => "X backend"))
        })
    }

    fn run(&self) -> Result<()> {
        slog_info!(self.log, "starting backend");
        Ok(())
    }

    fn stop(&self) {
        slog_info!(self.log, "stopping backend");

    }
}
