use std::{sync::mpsc::Receiver, thread};
use tungstenite::{Message, connect};

use crate::renderer::transform::Transform;

pub enum LocalUserUpdate {
    SendUserPosition(Transform),
}
pub enum UsersUpdate {
    SetUserPosition(Transform, u64),
}

pub fn start_user_handler(job_rx: Receiver<LocalUserUpdate>) {
    thread::spawn(move || {
        let mut user_id = 0;

        let (mut socket, _response) =
            connect("ws://localhost:42142/ws/user").expect("Can't connect");

        let _ = socket.send(Message::Binary(vec![1].into()));

        loop {
            if let Ok(job) = job_rx.recv() {
                match job {
                    LocalUserUpdate::SendUserPosition(transform) => {
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
                            println!("RECEIVED PLAYER DATA!? {}", data.len());
                            let player_amount = (data.len() - 1) / 28;
                            for i in 0..player_amount {
                                let player_id = u32::from_be_bytes([
                                    data[(i * 28) + 1],
                                    data[(i * 28) + 2],
                                    data[(i * 28) + 3],
                                    data[(i * 28) + 4],
                                ]);
                                if player_id == user_id {
                                    continue;
                                }
                                println!("player: {}", player_id);
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
