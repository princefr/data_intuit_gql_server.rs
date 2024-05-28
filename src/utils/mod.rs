#[cfg(test)]
pub mod Utils {
    #[cfg(test)]
    use std::sync::Arc;

    #[cfg(test)]
    use tokio::sync::{Mutex, RwLock};

    #[cfg(test)]
    use crate::{
        contexts::user_uid::UserUID, database::main::PostGreClient, firebase::main::Firebase,
        structs::user::User,
    };

    #[cfg(test)]
    pub async fn generate_testing_config(
        uuid: &str,
    ) -> Result<
        (
            Arc<Mutex<UserUID>>,
            Arc<Mutex<Option<User>>>,
            Arc<RwLock<PostGreClient>>,
            String,
        ),
        Box<dyn std::error::Error>,
    > {
        let firebase = Firebase::new().await;
        let custom_token = firebase
            .create_custom_token(uuid, false)
            .await
            .expect("Error creating custom token");
        let id_token = firebase
            .get_id_token(&custom_token)
            .await
            .expect("Error getting id token");
        let database = PostGreClient::new().await;
        let user_uid: Arc<Mutex<UserUID>> = Arc::new(Mutex::new(UserUID("".to_string())));
        let user: Arc<Mutex<Option<User>>> = Arc::new(Mutex::new(None));
        let database_rw = Arc::new(RwLock::new(database));
        Ok((user_uid, user, database_rw, id_token))
    }
}
