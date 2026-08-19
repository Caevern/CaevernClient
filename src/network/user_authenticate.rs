use std::{net::TcpStream, println};
use tungstenite::{Message, WebSocket, stream::MaybeTlsStream};

pub fn authenticate_user(
    mut socket: WebSocket<MaybeTlsStream<TcpStream>>,
) -> (WebSocket<MaybeTlsStream<TcpStream>>, u32) {
    println!("Connected to websocket /ws/user");

    let _ = socket.send(Message::Binary(vec![1].into()));

    while let Ok(message) = socket.read() {
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
