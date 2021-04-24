use super::user::User;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct Message {
  pub id: Uuid,
  pub user: User,
  pub text: String,
  pub created_at_utc: DateTime<Utc>,
}

impl Message {
  pub fn new(id: Uuid, user: User, text: &str, created_at_utc: DateTime<Utc>) -> Self{
    Message {
      id,
      user,
      text: String::from(text),
      created_at_utc
    }
  }
}