// https://github.com/kas-gui/kas/blob/c29c3bf59b8ce32bc1c753873f99d0ad9407a690/examples/layout.rs
// https://kas-gui.github.io/tutorials/counter.html
// https://github.com/kas-gui/kas/blob/c29c3bf59b8ce32bc1c753873f99d0ad9407a690/examples/counter.rs
// https://github.com/kas-gui/kas/blob/c29c3bf59b8ce32bc1c753873f99d0ad9407a690/examples/async-event.rs
use std::time::Instant;

use kas::widgets::StringLabel;
use lazy_static::lazy_static;
use regex::Regex;

use kas::prelude::*;
use kas::{
    impl_scope,
    prelude::EventMgr,
    widgets::{EditBox, TextButton},
    Widget,
};

use bedrock_cracker::{
    search_bedrock_pattern, world_seeds_from_bedrock_seed, Block, BlockType::BEDROCK,
};

lazy_static! {
    // the middle digit should never be negative
    // ^(\n?[-\d]+\s\d\s[-\d]+)+$ regex to match positions
    static ref POSITIONS_REGEX: Regex = Regex::new(r#"^([-\d]+\s\d\s[-\d]+\n?)+$"#).unwrap();
}

#[derive(Clone, Debug)]
struct Submit;

impl_scope! {
    #[widget{
        layout = list(down): ["Nether Bedrock Cracker", self.display, TextButton::new_msg("Submit", Submit), self.response];
    }]

    #[derive(Debug)]
    struct NetherBedRockCracker {
        core: widget_core!(),
        #[widget] display: EditBox,
        #[widget] response: StringLabel,
    }

    impl Self {
        fn new() -> Self {
            Self {
                core: Default::default(),
                display: EditBox::new("Place the position of bedrock located at y = 4 in \"x y z\" on each line").with_multi_line(true),
                response: StringLabel::new("....".to_string()),
            }
        }
    }

    // `impl Self` is equivalent to `impl Counter` here.
    // It's more useful when the type has generic parameters!
    impl Widget for Self {
        fn handle_message(&mut self, mgr: &mut EventMgr) {
            if let Some(Submit) = mgr.try_pop() {
                // read the data
                let contents = self.display.get_str();

                // ensure data is in the correct format
                if !POSITIONS_REGEX.is_match(&contents) {
                    *mgr |= self.response.set_string("Invalid input".into());
                    return;
                }

                // parse data
                let mut blocks: Vec<Block> = Vec::new();
                for position in contents.split("\n") {
                    // println!("{}", position);
                    if position.len() != 0 {
                        let mut position = position.split(" ");
                        let x = position.next().unwrap().parse::<i32>().unwrap();
                        let y = position.next().unwrap().parse::<i32>().unwrap();
                        let z = position.next().unwrap().parse::<i32>().unwrap();
                        blocks.push(Block::new(x, y, z, BEDROCK));
                    }
                }

                let start = Instant::now();

                // start cracking
                let mut response = String::from("Starting Cracking");
                *mgr |= self.response.set_string(response.clone());

                let (sender, receiver) = std::sync::mpsc::channel::<String>();
                let rx = search_bedrock_pattern(&mut blocks, 16);

                mgr.push_spawn(self.id(), async move {
                    for seed in rx {
                        let world_seeds = world_seeds_from_bedrock_seed(seed, true);
                        for world_seed in world_seeds {
                            sender.send(format!("Found World seed: {world_seed}\n"));
                        }
                    }
                });

                 loop {
                    match receiver.try_recv() {
                        Ok(message) => {
                            response.push_str(message.as_str());
                            *mgr |= self.response.set_string(response.clone());
                        },
                        Err(err) => {
                            match err {
                                std::sync::mpsc::TryRecvError::Empty => {
                                    // break
                                },
                                std::sync::mpsc::TryRecvError::Disconnected => {
                                    let execution_time = start.elapsed().as_secs();
                                    response.push_str(format!("Time elapsed: {execution_time}s").as_str());
                                    *mgr |= self.response.set_string(response.clone());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    impl Window for Self {
        fn title(&self) -> &str { "Nether Bedrock Cracker" }
    }
}

fn main() -> kas::shell::Result<()> {
    kas::shell::DefaultShell::new(kas::theme::FlatTheme::new())?
        .with(NetherBedRockCracker::new())?
        .run()
}
