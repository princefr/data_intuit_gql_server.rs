use std::sync::Arc;

use crate::{
    contexts::user_uid::UserUID,
    database::main::PostGreClient,
    guards::{auth::AuthTokenGuard, user::UserExistGuard},
    structs::user::User,
    traits::user::UserTrait,
};
use async_graphql::*;
use tokio::sync::{Mutex, RwLock};

pub struct Mutation;

#[Object]
impl Mutation {
    #[graphql(guard = "AuthTokenGuard")]
    async fn create_user<'ctx>(&self, ctx: &Context<'ctx>, input: User) -> Result<User, Error> {
        let _useruid = ctx.data::<Arc<Mutex<UserUID>>>().unwrap().lock().await;
        let database = ctx
            .data::<Arc<RwLock<PostGreClient>>>()
            .unwrap()
            .read()
            .await;
        let mut input = input;
        input.fill_id(_useruid.0.clone());
        Ok(database.create_user(&input).await?)
    }

    #[graphql(guard = "AuthTokenGuard.and(UserExistGuard)")]
    async fn update_user_name<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        user_name: String,
    ) -> Result<User, Error> {
        let database = ctx
            .data::<Arc<RwLock<PostGreClient>>>()
            .unwrap()
            .read()
            .await;
        let _useruid = ctx.data::<Arc<Mutex<UserUID>>>().unwrap().lock().await;
        Ok(database.update_user_name(&user_name, &_useruid.0).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexts::token::Token;
    use crate::firebase::main::Firebase;
    use crate::queries::main::Query;
    use crate::traits::user::UserTrait;
    use crate::utils::Utils;
    use async_graphql::Schema;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_user() {
        let uuid = Uuid::new_v4();
        let (user_uid, user, database_rw, token) =
            Utils::generate_testing_config(&uuid.clone().to_string())
                .await
                .unwrap();
        let schema = Schema::build(Query, Mutation, EmptySubscription)
            .data(user_uid)
            .data(user)
            .data(database_rw)
            .data(Token(format!("Bearer {}", token)))
            .data(Firebase::new().await)
            .finish();

        let query = Request::new(
            r#"
            mutation CreateUser($input: UserInput!){
               createUser(input: $input) {
                   id
                   name
                   email
               }
            }
            "#,
        )
        .variables(Variables::from_value(value!({
            "input": {
                "name": "Test User",
                "email": "blabal@gmail.com",
                "password": "password123456"
            }
        })));
        let executed_query = schema.execute(query).await;
        assert_eq!(executed_query.errors.first(), None);
        assert_eq!(
            executed_query.data,
            value!({"createUser": {"id": uuid.clone().to_string(), "name": "Test User", "email": "blabal@gmail.com"}})
        );
    }

    #[tokio::test]
    async fn test_update_user_name() {
        let uuid = Uuid::new_v4();
        let (user_uid, user, database_rw, token) =
            Utils::generate_testing_config(&uuid.to_string())
                .await
                .unwrap();
        database_rw
            .write()
            .await
            .create_test_user(&uuid.clone().to_string())
            .await
            .unwrap();
        let schema = Schema::build(Query, Mutation, EmptySubscription)
            .data(user_uid)
            .data(user)
            .data(database_rw)
            .data(Token(format!("Bearer {}", token)))
            .data(Firebase::new().await)
            .finish();

        let query = Request::new(
            r#"
            mutation UpdateUserName($userName: String!){
               updateUserName(userName: $userName) {
                   name
               }
            }
            "#,
        )
        .variables(Variables::from_value(value!({
            "userName": "Test User modified"
        })));
        let executed_query = schema.execute(query).await;
        assert_eq!(executed_query.errors.first(), None);
        assert_eq!(
            executed_query.data,
            value!({"updateUserName": {"name": "Test User modified"}})
        );
    }
}
