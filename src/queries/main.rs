use std::sync::Arc;

use crate::{
    guards::{auth::AuthTokenGuard, user::UserExistGuard},
    structs::user::User,
};
use async_graphql::*;
use tokio::sync::Mutex;

pub struct Query;

#[Object]
impl Query {
    #[graphql(guard = "AuthTokenGuard.and(UserExistGuard)")]
    async fn user<'ctx>(&self, ctx: &Context<'ctx>) -> Result<User, Error> {
        let user = ctx.data_unchecked::<Arc<Mutex<Option<User>>>>();
        let user = user.lock().await;
        let user_back = user.as_ref().unwrap();
        Ok(user_back.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexts::token::Token;
    use crate::firebase::main::Firebase;
    use crate::traits::user::UserTrait;
    use crate::utils::Utils;
    use async_graphql::{EmptyMutation, Schema};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_user() {
        let uuid = Uuid::new_v4();
        let (user_uid, user, database_rw, token) =
            Utils::generate_testing_config(&uuid.to_string())
                .await
                .unwrap();

        let user_db = database_rw
            .write()
            .await
            .create_test_user(&uuid.clone().to_string())
            .await
            .unwrap();

        let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
            .data(user_uid)
            .data(user)
            .data(database_rw)
            .data(Token(format!("Bearer {}", token)))
            .data(Firebase::new().await)
            .finish();

        let query = Request::new(
            r#"
            query {
                user {
                    id
                    name
                    email

                }
            }
            "#,
        );
        let res = schema.execute(query).await;
        assert_eq!(res.errors.first(), None);
        assert_eq!(
            res.data,
            value!({
                "user": {
                    "name": user_db.name,
                    "email": user_db.email,
                    "id": user_db.id,
                }
            })
        );
    }
}
