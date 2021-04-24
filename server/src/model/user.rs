use uuid::Uuid;

pub struct User {
    pub id: Uuid,
    pub name: String,
}

impl User {
    pub fn new(id: Uuid, name: &str) -> Self {
        User {
            id,
            name: String::from(name),
        }
    }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works(){
    let user = User::new(Uuid::new_v4(),  "Dao Lam");
    assert_eq!(user.name, "Dao Lam");
    assert_ne!(user.name, "Dao Vinh Lam");
  }
}