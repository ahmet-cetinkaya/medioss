use crate::bootstrap::report::fail;

/// Binds the server socket; a bind failure exits the process (untestable).
pub async fn bind_listener(host: std::net::IpAddr, port: u16) -> tokio::net::TcpListener {
    let addr = std::net::SocketAddr::new(host, port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|err| fail(format!("could not bind {addr}: {err}")));
    // Port 0 binds an ephemeral port, so only the bound address is informative.
    match listener.local_addr() {
        Ok(bound) => tracing::info!("medioss listening on {bound}"),
        Err(err) => tracing::warn!("medioss listening on {addr}, bound address unavailable: {err}"),
    }
    listener
}

#[cfg(test)]
#[path = "listener.tests.rs"]
mod tests;
