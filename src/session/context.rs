use crate::ai::client::{Message, Role};
use crate::ai::prompt;
use std::path::PathBuf;

pub struct Session {
    system_prompt: Message,
    messages: Vec<Message>,
    max_messages: usize,
    current_dir: PathBuf,
}

impl Session {
    pub fn new(max_messages: usize, system_prompt: impl Into<String>) -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            system_prompt: Message::system(system_prompt.into()),
            messages: Vec::new(),
            max_messages,
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
        if self.messages.len() > self.max_messages {
            // 保持消息数量在 max_messages 内
            self.messages
                .drain(0..self.messages.len() - self.max_messages);
        }
    }

    pub fn get_messages(&self) -> Vec<Message> {
        let mut out = Vec::with_capacity(self.messages.len() + 1);
        out.push(self.system_prompt.clone());
        out.extend(self.messages.iter().cloned());
        out
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::client::{Message, Role};

    // 1. 新建 session 后，messages 应为空
    #[test]
    fn new_session_has_no_messages() {
        let session = Session::new(5, "System prompt");
        assert_eq!(session.get_messages().len(), 1); // 包含 system_prompt
    }

    // 2. add_message 后，get_messages 长度应增加
    #[test]
    fn add_message_increases_count() {
        let mut session = Session::new(5, "System prompt");
        let message = Message {
            role: Role::User,
            content: "Hello".to_string(),
        };
        session.add_message(message);
        assert_eq!(session.get_messages().len(), 2); // 包含 system_prompt
    }

    // 3. clear 后，messages 应清空
    #[test]
    fn clear_removes_all_messages() {
        let mut session = Session::new(5, "System prompt");
        session.add_message(Message {
            role: Role::User,
            content: "Hello".to_string(),
        });
        session.clear();
        assert!(session.get_messages().is_empty());
    }

    // 4. change_dir 后，current_dir 应更新
    #[test]
    fn change_dir_updates_current_dir() {
        let mut session = Session::new(5, "System prompt");
        let new_dir = PathBuf::from("/tmp");
        session.change_dir(new_dir.clone());
        assert_eq!(session.current_dir(), &new_dir);
    }

    // 5. ⭐ 关键测试：当消息超过 max_messages 时，应触发滑动窗口
    //    比如 max_messages=2，连续 add 6 条 user 消息后，应该只剩 2 条（drain 了 0..4）
    #[test]
    fn sliding_window_drops_old_messages() {
        let mut session = Session::new(2, "System prompt");
        for i in 0..6 {
            session.add_message(Message {
                role: Role::User,
                content: format!("Message {}", i),
            });
        }
        assert_eq!(session.get_messages().len(), 3); // 包含 system_prompt
    }
}
