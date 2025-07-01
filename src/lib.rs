use std::io;
use std::{collections::HashMap, io::Write};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}

impl Body {
    pub fn command(&self) -> &str {
        match self.payload {
            Payload::Init { .. } => "init",
            Payload::Echo { .. } => "echo",
            _ => panic!("Not a recognised command"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Payload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {/* empty */},
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
}

pub trait Handler {
    fn name(&self) -> &str;
    fn handle(&self, message: Message) -> Message;
}

pub struct Node {
    handlers: HashMap<String, Box<dyn Handler>>,
}

impl Node {
    pub fn register(&mut self, handler: Box<dyn Handler>) {
        let name = handler.name().into();
        self.handlers.insert(name, handler);
    }

    pub fn run(self) {
        println!("Starting Node...");
        let mut buffer = String::new();
        let stdin = io::stdin();
        loop {
            buffer.clear();
            match stdin.read_line(&mut buffer) {
                Ok(0) => break,
                Ok(_) => {
                    let line = buffer.trim();
                    let message = serde_json::from_str::<Message>(line);
                    match message {
                        Ok(message) => {
                            let command = message.body.command();
                            if let Some(handler) = self.handlers.get(command) {
                                let response = handler.handle(message);
                                let mut stdout = io::stdout();
                                stdout
                                    .write_all(
                                        serde_json::to_string(&response)
                                            .expect(
                                                "At least serializing should always work, right?",
                                            )
                                            .as_bytes(),
                                    )
                                    .unwrap();
                                stdout.write_all(b"\n").unwrap();
                            } else {
                                eprintln!("Hanlder not found for command: {}", &command);
                            }
                        }
                        Err(error) => {
                            eprintln!("Failed to parse JSON: {}", error);
                        }
                    }
                }
                Err(_) => {
                    panic!("Lmao you're on your own here");
                }
            }
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        let init_handler = Box::new(InitHandler);
        let mut node = Self {
            handlers: HashMap::new(),
        };
        node.register(init_handler);
        node
    }
}

struct InitHandler;
impl Handler for InitHandler {
    fn name(&self) -> &str {
        "init"
    }

    fn handle(&self, message: Message) -> Message {
        Message {
            src: message.dst,
            dst: message.src,
            body: Body {
                id: message.body.id,
                in_reply_to: message.body.id,
                payload: Payload::InitOk {},
            },
        }
    }
}
