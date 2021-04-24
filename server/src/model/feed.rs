use super::message::Message;
pub struct Feed {
  pub messages: Vec<Message>
}

impl Feed {
  pub fn add_message(&mut self, message: Message) {
    self.messages.push(message);
    self.messages.sort_by_key(|message| message.created_at_utc)
  }
}

