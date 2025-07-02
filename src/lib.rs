use core::panic;
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
    fn handle(&self, state: &mut NodeState, message: Message) -> Message;
}

#[derive(Default)]
pub struct NodeState {
    node_id: String,
    neighbours: Vec<String>,
}

impl NodeState {
    pub fn set_node_id(&mut self, id: String) {
        eprintln!("I am node {}", &id);
        self.node_id = id;
    }

    pub fn set_node_ids(&mut self, node_ids: Vec<String>) {
        eprintln!("My neighbours are {:?}", &node_ids);
        self.neighbours = node_ids;
    }
}

pub struct Node {
    handlers: HashMap<String, Box<dyn Handler>>,
    state: NodeState,
}

impl Node {
    pub fn register(&mut self, handler: Box<dyn Handler>) {
        let name = handler.name().into();
        self.handlers.insert(name, handler);
    }

    pub fn run(&mut self) {
        eprintln!("Starting Node...");
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
                                let response = handler.handle(&mut self.state, message);
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
            state: NodeState::default(),
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

    fn handle(&self, state: &mut NodeState, message: Message) -> Message {
        if let Payload::Init { node_id, node_ids } = message.body.payload {
            state.set_node_id(node_id);
            state.set_node_ids(node_ids);
            Message {
                src: message.dst,
                dst: message.src,
                body: Body {
                    id: message.body.id,
                    in_reply_to: message.body.id,
                    payload: Payload::InitOk {},
                },
            }
        } else {
            panic!("Weird init message");
        }
    }
}
