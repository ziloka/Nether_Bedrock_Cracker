// https://github.com/emilk/egui/blob/master/examples/hello_world/src/main.rs
// https://github.com/emilk/egui/issues/110

use std::{
    sync::mpsc::{Receiver, Sender},
    time::Instant,
};

use eframe::egui;

#[allow(unused_imports)]
use bedrock_cracker::{
    search_bedrock_pattern, world_seeds_from_bedrock_seed, Block,
    BlockType::{BEDROCK, OTHER},
};
use egui::ScrollArea;

fn main() -> Result<(), eframe::Error> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    // tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}

struct MyApp {
    contents: String,
    response: String,
    cracker_tx: Sender<Message>,
    cracker_rx: Receiver<Message>,
}
enum Message {
    Start(String),
    Regular(String),
    Disconnect(String),
}

impl Default for MyApp {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<Message>();
        Self {
            contents: "Contents".to_owned(),
            response: "".to_owned(),
            cracker_tx: tx,
            cracker_rx: rx,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Nether Bedrock Cracker");
            let contents_label = ui.label("positions");

            ui.vertical(|ui| {
                ui.set_max_height(200.0);
                ScrollArea::vertical().show(ui, |ui| {
                    ui.text_edit_multiline(&mut self.contents)
                        .labelled_by(contents_label.id);
                });
            });

            if ui.button("Submit").clicked() {
                let mut blocks: Vec<Block> = Vec::new();
                for position in self.contents.split("\n") {
                    if position.len() != 0 {
                        let mut position = position.split(" ");
                        let x = position.next().unwrap().parse::<i32>().unwrap();
                        let y = position.next().unwrap().parse::<i32>().unwrap();
                        let z = position.next().unwrap().parse::<i32>().unwrap();
                        blocks.push(Block::new(x, y, z, BEDROCK));
                    }
                }

                let sender = self.cracker_tx.clone();
                std::thread::spawn(move || {
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

            if let Ok(message) = self.cracker_rx.try_recv() {
                match message {
                    Message::Start(message) => self.response = message,
                    Message::Regular(message) => self.response += message.as_str(),
                    Message::Disconnect(message) => self.response += message.as_str(),
                }
            }

            ui.label(self.response.as_str());
        });
    }
}
