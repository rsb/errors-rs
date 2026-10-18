use tracing::{debug, warn};

pub fn a() {
    warn!("degraded");
    debug!("fine");
}

pub fn b() {
    tracing::error!("failed");
}
