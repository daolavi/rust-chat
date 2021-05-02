use super::message::Message;

#[derive(Default)]
pub struct Feed {
  pub messages: Vec<Message>
}

impl Feed {
  pub fn add_message(&mut self, message: Message) {
    self.messages.push(message);
    self.messages.sort_by_key(|message| message.created_at_utc)
  }

  pub fn iter(&self) -> impl Iterator<Item = &Message> {
    self.messages.iter()
  }
}

