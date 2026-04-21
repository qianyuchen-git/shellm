use crate::ai::client::Message;

pub struct Session{
    messages: Vec<Message>,
    max_turns: usize,
}

impl Session {
    pub fn new(max_turns: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_turns,
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        if self.messages.len() > self.max_turns * 2 {
            // 保持消息数量在 max_turns 内（每轮包含用户和系统消息）
            self.messages.drain(1..3);
        }
    }

    pub fn get_messages(&self) -> &[Message] {
        &self.messages
    }
}