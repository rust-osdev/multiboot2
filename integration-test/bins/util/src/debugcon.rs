//! Driver for QEMU's debugcon device.

use core::fmt::{Arguments, Write};
use log::{LevelFilter, Log, Metadata, Record};

static LOGGER: DebugconLogger = DebugconLogger;

struct Debugcon;

/// Internal API for the `println!` macro.
pub fn _print(args: Arguments) {
    Debugcon.write_fmt(args).unwrap();
}

impl Debugcon {
    /// I/O port of QEMUs debugcon device on x86.
    const IO_PORT: u16 = 0xe9;

    pub fn write_byte(byte: u8) {
        unsafe {
            core::arch::asm!(
                "outb %al, %dx",
                in("al") byte,
                in("dx") Self::IO_PORT,
                options(att_syntax, nomem, nostack, preserves_flags)
            )
        }
    }
}

impl Write for Debugcon {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for &byte in s.as_bytes() {
            Debugcon::write_byte(byte);
        }
        Ok(())
    }
}

pub struct DebugconLogger;

impl DebugconLogger {
    pub fn init() {
        // Ignore, as we can't do anything about it here.
        let _ = log::set_logger(&LOGGER);
        log::set_max_level(LevelFilter::Trace);
    }
}

impl Log for DebugconLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        // Ignore result as we can't do anything about it.
        let _ = writeln!(
            Debugcon,
            "[{:>5}: {}@{}]: {}",
            record.level(),
            record.file().unwrap_or("<unknown>"),
            record.line().unwrap_or(0),
            record.args()
        );
    }

    fn flush(&self) {}
}
