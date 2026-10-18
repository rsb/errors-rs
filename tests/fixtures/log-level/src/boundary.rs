pub fn execute(command: &str) {
    if let Err(e) = crate::run(command) {
        tracing::error!(command, error = %e, "command failed");
        tracing::warn!("degraded");
    }
}
