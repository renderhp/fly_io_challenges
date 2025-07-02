use maelstrom::{Body, Handler, Message, Node, NodeState, Payload};

#[derive(Default)]
struct EchoHandler;
impl Handler for EchoHandler {
    fn name(&self) -> &str {
        "echo"
    }

    fn handle(&self, _state: &mut NodeState, message: Message) -> Message {
        let echo_value = match message.body.payload {
            Payload::Echo { echo } => echo,
            _ => panic!("Expected echo payload"),
        };
        Message {
            src: message.dst,
            dst: message.src,
            body: Body {
                id: message.body.id,
                in_reply_to: message.body.id,
                payload: Payload::EchoOk { echo: echo_value },
            },
        }
    }
}

fn main() {
    let mut node = Node::default();
    let echo_handler = Box::new(EchoHandler);
    node.register(echo_handler);
    node.run();
}
