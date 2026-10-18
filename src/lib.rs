//! What every rsb error enum shares: the location each layer records, the
//! switch that shows those locations, and the panic capture the dispatcher
//! reads. The rules are `rules/standards/errors.md` and `panics.md`.
//!
//! An error enum uses `Loc` as its implicit field:
//!
//! ```
//! use snafu::prelude::*;
//! use rsb_errors::Loc;
//!
//! #[derive(Debug, Snafu)]
//! pub enum Error {
//!     #[snafu(display("open raw{location}: {source}"))]
//!     OpenRaw { source: std::io::Error, #[snafu(implicit)] location: Loc },
//! }
//! ```
//!
//! `{location}` prints nothing until the binary calls `show_locations(true)`,
//! after which it prints ` (at file:line)`.
//!
//! The `check` feature adds [`check::run`], the repository checks of the
//! standards' Check sections, for each consumer's `cargo xtask errors`.

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "check")]
pub mod check;
pub mod panic;

static SHOW_LOCATIONS: AtomicBool = AtomicBool::new(false);

/// Show ` (at file:line)` after each layer of every error line. A binary
/// calls this once at startup from its `--debug` flag or
/// `<APP>_ERROR_LOCATIONS` variable. Off by default.
pub fn show_locations(on: bool) {
    SHOW_LOCATIONS.store(on, Ordering::Relaxed);
}

/// Whether error lines currently show locations.
pub fn locations_shown() -> bool {
    SHOW_LOCATIONS.load(Ordering::Relaxed)
}

/// Where an error was built: the `.context(...)`, `.fail()` or `ensure!`
/// that created it. Captured by snafu through `#[track_caller]`; the
/// variant declares `#[snafu(implicit)] location: Loc` and writes nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Loc(&'static std::panic::Location<'static>);

impl Loc {
    /// The source file, as the compiler recorded it.
    pub fn file(&self) -> &'static str {
        self.0.file()
    }

    /// The line within `file`.
    pub fn line(&self) -> u32 {
        self.0.line()
    }
}

impl snafu::GenerateImplicitData for Loc {
    #[track_caller]
    fn generate() -> Self {
        Loc(std::panic::Location::caller())
    }
}

impl fmt::Display for Loc {
    /// Prints ` (at file:line)` when locations are shown, and nothing
    /// otherwise, so a display string can always say `"open raw{location}"`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if locations_shown() {
            write!(f, " (at {}:{})", self.0.file(), self.0.line())
        } else {
            Ok(())
        }
    }
}
