use crate::{contexts::user_uid::UserUID, database, structs::user::User, traits::user::UserTrait};
use async_graphql::*;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub struct UserExistGuard;

impl UserExistGuard {}

impl Guard for UserExistGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let _useruid = ctx.data::<Arc<Mutex<UserUID>>>()?;
        let f = _useruid.as_ref().lock().await;
        let database = ctx
            .data::<Arc<RwLock<database::main::PostGreClient>>>()
            .unwrap();
        let user = database.as_ref().read().await.get_user(&f.0).await?;

        let _ = ctx
            .data_unchecked::<Arc<Mutex<Option<User>>>>()
            .lock()
            .await
            .insert(user);
        Ok(())
    }
}
