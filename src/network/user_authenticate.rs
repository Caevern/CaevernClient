use std::println;
use futures_util::{StreamExt, SinkExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::Message;

pub async fn authenticate_user(
    mut socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
) -> (WebSocketStream<MaybeTlsStream<TcpStream>>, u32) {
    println!("Connected to websocket /ws/user");

    let _ = socket.send(Message::Binary(vec![1].into())).await;

    while let Some(message) = socket.next().await {
        let message = message.unwrap();
        let data = message.into_data();
        if data.len() > 0 {
            match data[0] {
                2 => {
                    let user_id = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
                    println!("Received Free ID {}", user_id);

                    return (socket, user_id);
                }
                _ => {
                    println!("Unknown Command ({})", data[0]);
                }
            }
        }
    }

    return (socket, 0);
}
