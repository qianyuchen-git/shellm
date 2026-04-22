use crate::ai::client::Message;
use std::path::PathBuf;

pub struct Session {
    messages: Vec<Message>,
    max_turns: usize,
    current_dir: PathBuf,
}

impl Session {
    pub fn new(max_turns: usize) -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            messages: Vec::new(),
            max_turns,
            current_dir,
        }
    }

    pub fn current_dir(&self) -> &PathBuf {
        &self.current_dir
    }

    pub fn change_dir(&mut self, new_dir: PathBuf) {
        self.current_dir = new_dir;
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

    pub fn clear(&mut self) {
        self.messages.clear();
    }
}
