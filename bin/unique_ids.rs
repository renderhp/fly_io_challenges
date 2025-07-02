use std::time::{SystemTime, UNIX_EPOCH};

use maelstrom::{Body, Handler, Message, Node, NodeState, Payload};

#[derive(Default)]
struct GenerateHandler;
impl Handler for GenerateHandler {
    fn name(&self) -> &str {
        "generate"
    }

    fn handle(&self, state: &mut NodeState, message: Message) -> Message {
        let node_id = &state.node_id;
        let time_micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros();
        Message {
            src: node_id.clone(),
            dst: message.src,
            body: Body {
                id: message.body.id,
                in_reply_to: message.body.id,
                payload: Payload::GenerateOk {
                    id: format!("{}_{}", node_id, time_micros),
                },
            },
        }
    }
}

fn main() {
    let mut node = Node::default();
    let echo_handler = Box::new(GenerateHandler);
    node.register(echo_handler);
    node.run();
}
