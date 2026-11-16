use reqwest::Client;
use std::{sync::mpsc::Sender, thread};

use crate::{
    game::engine_command::EngineCommand,
    game_objects::{scene::Scene, world::World},
};

pub fn get_world(engine_command_tx: Sender<EngineCommand>, server_url: String, world_id: u32) {
    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

        runtime.block_on(async move {
            let client = Client::new();

            match load_world_config(&client, &server_url, world_id).await {
                Ok(scene) => {
                    println!("Loading scene: {}", scene.name);

                    let mut world = World::new();
                    world.load_from_scene(scene);
                    engine_command_tx
                        .send(EngineCommand::LoadWorldData(Box::new(world)))
                        .ok();
                }
                Err(err) => eprintln!("Failed to load world: {err}"),
            }
        });
    });
}

async fn load_world_config(
    client: &Client,
    server_url: &str,
    world_id: u32,
) -> Result<Scene, String> {
    let url = format!("http://{server_url}/worlds/{world_id}/world.json");
    println!("Loading world from: {url}");

    let scene: Scene = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    Ok(scene)
}
