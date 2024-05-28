#[derive(Clone)]
pub struct UserUID(pub String);

impl UserUID {
    pub fn update(&mut self, user_uid: String) {
        self.0 = user_uid;
    }
}

impl From<UserUID> for uuid::Uuid {
    fn from(user_uid: UserUID) -> Self {
        uuid::Uuid::parse_str(&user_uid.0).unwrap()
    }
}
