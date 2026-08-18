use cgmath::Vector3;
use std::{
    collections::HashMap,
    f32, println,
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
        let mut users_loaded = HashMap::new();

        let (mut socket, _response) =
            connect("ws://localhost:42142/ws/user").expect("Can't connect");
        println!("Connected to websocket /ws/user");

        let _ = socket.send(Message::Binary(vec![1].into()));

        loop {
            if let Ok(mut job) = data_thread_rx.recv() {
                match job {
                    UserUpdate::SendUserPosition(_) => {
                        while let Ok(newer_job) = data_thread_rx.try_recv() {
                            job = newer_job;
                        }
                    }
                    _ => (),
                }

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

                    UserUpdate::UpdateAvatarId(temp_user_id, object_id) => {
                        users_loaded.insert(temp_user_id, object_id);
                    }

                    UserUpdate::SendReadySignal => {
                        let _ = socket.send(Message::Binary(vec![2].into()));
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
                        }
                        3 => {
                            let player_amount = (data.len() - 1) / 28;

                            for i in 0..player_amount {
                                let player_id = u32::from_be_bytes([
                                    data[(i * 28) + 1],
                                    data[(i * 28) + 2],
                                    data[(i * 28) + 3],
                                    data[(i * 28) + 4],
                                ]);

                                let transform = Transform::new(
                                    Vector3::new(
                                        f32::from_be_bytes([
                                            data[(i * 28) + 5],
                                            data[(i * 28) + 6],
                                            data[(i * 28) + 7],
                                            data[(i * 28) + 8],
                                        ]),
                                        f32::from_be_bytes([
                                            data[(i * 28) + 9],
                                            data[(i * 28) + 10],
                                            data[(i * 28) + 11],
                                            data[(i * 28) + 12],
                                        ]),
                                        f32::from_be_bytes([
                                            data[(i * 28) + 13],
                                            data[(i * 28) + 14],
                                            data[(i * 28) + 15],
                                            data[(i * 28) + 16],
                                        ]),
                                    ),
                                    Vector3::new(
                                        f32::from_be_bytes([
                                            data[(i * 28) + 17],
                                            data[(i * 28) + 18],
                                            data[(i * 28) + 19],
                                            data[(i * 28) + 20],
                                        ]),
                                        f32::from_be_bytes([
                                            data[(i * 28) + 21],
                                            data[(i * 28) + 22],
                                            data[(i * 28) + 23],
                                            data[(i * 28) + 24],
                                        ]) - f32::consts::PI,
                                        f32::from_be_bytes([
                                            data[(i * 28) + 25],
                                            data[(i * 28) + 26],
                                            data[(i * 28) + 27],
                                            data[(i * 28) + 28],
                                        ]),
                                    ),
                                    Vector3::new(1.0, 1.0, 1.0),
                                );

                                if !users_loaded.contains_key(&player_id) {
                                    users_loaded.insert(player_id, 0);
                                    println!("Registering user with id: {}", player_id);
                                    avatar_thread_tx
                                        .send(AvatarUpdate::RegisterUser(transform, player_id))
                                        .expect(
                                            "Registering user with game has FAILED (somehow...)",
                                        );
                                } else if users_loaded.contains_key(&player_id) {
                                    let object_id = *users_loaded.get(&player_id).unwrap();
                                    if object_id == 0 {
                                        continue;
                                    }

                                    avatar_thread_tx
                                        .send(AvatarUpdate::SetUserPosition(
                                            transform,
                                            object_id,
                                        ))
                                        .expect(
                                            "Setting avatar position in game thread has FAILED (somehow...)",
                                        );
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
                println!("Socket has closed, removed client: {}", user_id);
            }
        }
    });
}
