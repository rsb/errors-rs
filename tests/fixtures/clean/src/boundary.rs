pub fn serve(command: &str) {
    match crate::dispatch::execute(command) {
        Ok(()) => tracing::info!(command, "done"),
        Err(e) => tracing::error!(command, code = %e.code(), error = %e, "command failed"),
    }
    tracing::warn!("the boundary may warn");
}
