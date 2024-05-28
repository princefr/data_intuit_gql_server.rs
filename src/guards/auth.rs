use std::sync::Arc;

use async_graphql::*;
use tokio::sync::Mutex;

use crate::{
    contexts::{token::Token, user_uid::UserUID},
    firebase::main::Firebase,
};

pub struct AuthTokenGuard;

impl AuthTokenGuard {}

impl Guard for AuthTokenGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let token = ctx
            .data::<Token>()
            .map_err(|_| Error::new("Auth::Unauthorized"))?;
        let token = token.clone();
        let firebase = ctx.data::<Firebase>()?;
        let user_uid = ctx.data::<Arc<Mutex<UserUID>>>()?;

        let token = token.0;
        if token.is_empty() {
            return Err(Error::new("Auth::Unauthorized"));
        }

        let token = &token.replace("Bearer ", "");
        match firebase.verify_id_token(token).await {
            Ok(uid) => {
                let _ = user_uid.lock().await.update(uid);
                return Ok(());
            }
            Err(_) => return Err(Error::new("Auth::Unauthorized")),
        }
    }
}
