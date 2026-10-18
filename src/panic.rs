//! Capture where a panic happened so the dispatcher's `catch_unwind` can
//! report it as `Internal` with a file:line. See `rules/standards/panics.md`.
//!
//! ```no_run
//! rsb_errors::panic::install();
//! let result = std::panic::catch_unwind(|| { /* run the command */ });
//! if result.is_err() {
//!     let detail = rsb_errors::panic::take()
//!         .map(|p| p.to_string())
//!         .unwrap_or_else(|| "panic with no recorded location".to_owned());
//!     // Err(InternalSnafu { what: detail }.build())
//! }
//! ```

use std::cell::RefCell;
use std::fmt;
use std::panic::PanicHookInfo;

/// What the panic hook saw, for the catch site to pick up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// The panic message: the `expect` text, the `assert!` message, or
    /// std's own text for an index out of bounds or an overflow.
    pub message: String,
    /// Source file of the panic, or `<unknown>`.
    pub file: String,
    /// Line within `file`, or 0.
    pub line: u32,
    /// Column within `line`, or 0.
    pub column: u32,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {}:{}:{}",
            self.message, self.file, self.line, self.column
        )
    }
}

thread_local! {
    // The hook runs on the panicking thread and catch_unwind returns on
    // that same thread, so one slot per thread needs no locking.
    static LAST_PANIC: RefCell<Option<Report>> = const { RefCell::new(None) };
}

fn record(info: &PanicHookInfo<'_>) {
    let message = info
        .payload_as_str()
        .unwrap_or("panic payload was not a string")
        .to_owned();
    let (file, line, column) = match info.location() {
        Some(loc) => (loc.file().to_owned(), loc.line(), loc.column()),
        None => ("<unknown>".to_owned(), 0, 0),
    };
    let report = Report {
        message,
        file,
        line,
        column,
    };
    // try_with and try_borrow_mut: the hook itself must never panic.
    let _stored = LAST_PANIC.try_with(|slot| {
        if let Ok(mut slot) = slot.try_borrow_mut() {
            *slot = Some(report);
        }
    });
}

/// Install the recording hook once, at startup. The previous hook still
/// runs, so stderr shows the panic as before.
pub fn install() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        record(info);
        previous(info);
    }));
}

/// The last panic recorded on this thread, cleared by this call. The catch
/// site calls it after `catch_unwind` returns `Err`.
pub fn take() -> Option<Report> {
    LAST_PANIC.with(|slot| slot.borrow_mut().take())
}
