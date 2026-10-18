use tracing::instrument;

#[instrument(err)]
pub fn a() -> Result<(), E> { Ok(()) }

#[tracing::instrument(level = "debug", err)]
pub fn b() -> Result<(), E> { Ok(()) }

#[instrument(err(Debug), skip(x))]
pub fn c(x: u8) -> Result<(), E> { Ok(()) }

#[instrument(skip(x))]
pub fn d(x: u8) -> Result<(), E> { Ok(()) }

#[instrument(
    level = "debug",
    err
)]
pub fn e() -> Result<(), E> { Ok(()) }
