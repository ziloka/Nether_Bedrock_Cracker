use std::sync::mpsc::channel;
use std::thread;
use std::time::Instant;

use macroquad::prelude::*;

use macroquad::ui::{
    hash, root_ui,
    widgets,
};

use bedrock_cracker::{
    search_bedrock_pattern, world_seeds_from_bedrock_seed, Block,
    BlockType::BEDROCK,
};

enum Message {
    Start(String),
    Regular(String),
    Disconnect(String),
}

#[macroquad::main("UI showcase")]
async fn main() {

    let mut contents = String::new();
    let mut response = String::new();
    let (tx, rx) = channel::<Message>();

    loop {
        clear_background(WHITE);
        widgets::Window::new(hash!(), vec2(470., 50.), vec2(300., 300.))
            .label("Megaui Showcase Window")
            .ui(&mut *root_ui(), |ui| {
                ui.label(None, "This is editbox!");
                ui.editbox(hash!(), vec2(285., 165.), &mut contents);
                if ui.button(None, "Submit") {
                    let mut blocks: Vec<Block> = Vec::new();
                    for position in contents.split("\n") {
                        if position.len() != 0 {
                            let mut position = position.split(" ");
                            let x = position.next().unwrap().parse::<i32>().unwrap();
                            let y = position.next().unwrap().parse::<i32>().unwrap();
                            let z = position.next().unwrap().parse::<i32>().unwrap();
                            blocks.push(Block::new(x, y, z, BEDROCK));
                        }
                    }

                    if blocks.len() != 0 {
                        let sender = tx.clone();
                        thread::spawn(move || {
                            let start = Instant::now();
                            let rx = search_bedrock_pattern(&mut blocks, num_cpus::get() as u64);

                            sender.send(Message::Start("Started Cracking\n".to_string()));

                            for seed in rx {
                                let world_seeds = world_seeds_from_bedrock_seed(seed, true);
                                for world_seed in world_seeds {
                                    sender.send(Message::Regular(format!(
                                        "Found World seed: {world_seed}\n"
                                    )));
                                }
                            }
                            let execution_time = start.elapsed().as_secs();
                            sender.send(Message::Disconnect(format!(
                                "Time elapsed: {execution_time}s"
                            )));
                        });
                    }
                }

                if let Ok(message) = rx.try_recv() {
                    match message {
                        Message::Start(message) => response = message,
                        Message::Regular(message) => response += message.as_str(),
                        Message::Disconnect(message) => response += message.as_str(),
                    }
                }

                ui.label(None, response.as_str());
            });
        next_frame().await;
    }
}
