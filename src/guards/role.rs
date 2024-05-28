use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use async_graphql::*;

use crate::{
    contexts::user_uid::UserUID, database::main::PostGreClient, enums::role::Role,
    traits::user::UserTrait,
};

pub struct RoleGuard {
    roles: Role,
}

impl RoleGuard {
    pub fn new(roles: Role) -> Self {
        Self { roles }
    }
}

impl Guard for RoleGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let _useruid = ctx.data::<Arc<Mutex<UserUID>>>()?;
        let f = _useruid.as_ref().lock().await;
        let database = ctx
            .data::<Arc<RwLock<PostGreClient>>>()
            .unwrap()
            .read()
            .await;
        let roles: Vec<Role> = database.get_user_roles(&f.0).await?;

        if roles.contains(&self.roles) {
            Ok(())
        } else {
            Err(Error::new("Role::Unauthorized"))
        }
    }
}
