use std::{
    collections::HashSet,
    println,
    sync::mpsc::{Receiver, Sender},
    thread,
};
use tungstenite::{Message, connect};

use crate::{
    network::{avatar_updates::AvatarUpdate, user_updates::UserUpdate},
    renderer::transform::Transform,
};

pub fn start_user_handler(
    data_thread_rx: Receiver<UserUpdate>,
    avatar_thread_tx: Sender<AvatarUpdate>,
) {
    thread::spawn(move || {
        let mut user_id = 0;
        let mut users_loaded = HashSet::new();

        let (mut socket, _response) =
            connect("ws://localhost:42142/ws/user").expect("Can't connect");

        let _ = socket.send(Message::Binary(vec![1].into()));

        loop {
            if let Ok(job) = data_thread_rx.recv() {
                match job {
                    UserUpdate::SendUserPosition(transform) => {
                        let mut data_sending = vec![8];
                        for byte in transform.position.x.to_be_bytes() {
                            data_sending.push(byte);
                        }
                        for byte in transform.position.y.to_be_bytes() {
                            data_sending.push(byte);
                        }
                        for byte in transform.position.z.to_be_bytes() {
                            data_sending.push(byte);
                        }
                        for byte in transform.rotation.x.to_be_bytes() {
                            data_sending.push(byte);
                        }
                        for byte in transform.rotation.y.to_be_bytes() {
                            data_sending.push(byte);
                        }
                        for byte in transform.rotation.z.to_be_bytes() {
                            data_sending.push(byte);
                        }

                        let _ = socket.send(Message::Binary(data_sending.into()));
                    }
                }
            }

            if let Ok(result) = socket.read() {
                let data = result.into_data();
                if data.len() > 0 {
                    match data[0] {
                        2 => {
                            user_id = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
                            println!("Received Free ID {}", user_id);
                            let _ = socket.send(Message::Binary(vec![2].into()));
                        }
                        3 => {
                            println!("Received {} bytes of player data", data.len());
                            let player_amount = (data.len() - 1) / 28;
                            println!("player count: {}", player_amount);
                            for i in 0..player_amount {
                                let player_id = u32::from_be_bytes([
                                    data[(i * 28) + 1],
                                    data[(i * 28) + 2],
                                    data[(i * 28) + 3],
                                    data[(i * 28) + 4],
                                ]);
                                println!("Player: {}", player_id);

                                if !users_loaded.contains(&player_id) {
                                    users_loaded.insert(player_id);
                                    avatar_thread_tx.send(AvatarUpdate::RegisterUser(
                                        Transform::zero(),
                                        player_id,
                                    ));
                                    println!("Registering user with id: {}", player_id);
                                }
                            }
                            let _ = socket.send(Message::Binary(vec![2].into()));
                        }
                        0 => {}
                        _ => {
                            println!("Unknown Command ({})", data[0]);
                        }
                    }
                }
            } else {
                println!("Socket has closed!")
            }
        }
    });
}
