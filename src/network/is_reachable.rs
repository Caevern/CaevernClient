use std::time::Duration;
use tokio::net::TcpStream;

pub async fn is_server_reachable(server_addr: &str) -> bool {
    let addr = if server_addr.contains(':') {
        server_addr.to_string()
    } else {
        format!("{}:5000", server_addr)
    };

    match tokio::time::timeout(Duration::from_secs(3), TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => true,
        _ => false,
    }
}
