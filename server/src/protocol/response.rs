use uuid::Uuid;

pub struct ResponseParcel {
  pub client_id: Uuid,
  pub respose: Response
}

pub enum Response {
  Error,
  Alive,
  Joined,
  UserJoined,
  UserLeft,
  Posted,
  UserPosted 
}

pub struct Error{
  
}