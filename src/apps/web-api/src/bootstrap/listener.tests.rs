use super::bind_listener;

#[tokio::test]
async fn bind_listener_on_port_zero_binds_an_ephemeral_port_on_the_host() {
    let host: std::net::IpAddr = "127.0.0.1".parse().unwrap();

    let listener = bind_listener(host, 0).await;

    let bound = listener.local_addr().unwrap();
    assert_eq!(bound.ip(), host);
    assert_ne!(bound.port(), 0);
}
