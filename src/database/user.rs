use super::main::PostGreClient;
use crate::enums::role::Role;
use crate::structs::user::User;
use crate::traits::user::UserTrait;
use tokio_postgres::Error;

impl UserTrait for PostGreClient {
    async fn get_user_roles<'a>(&self, user_uid: &'a str) -> Result<Vec<Role>, Error> {
        let rows = self
            .client
            .query(
                "SELECT role FROM roles WHERE firebase_uid = $1",
                &[&user_uid],
            )
            .await?;
        let mut roles = Vec::new();
        for row in rows {
            roles.push(row.get(0));
        }
        Ok(roles)
    }

    async fn save_user_role<'a>(&self, user_uid: &'a str, role: &'a Role) -> Result<(), Error> {
        self.client
            .execute(
                "INSERT INTO roles (firebase_uid, role) VALUES ($1, $2)",
                &[&user_uid, &role],
            )
            .await?;
        Ok(())
    }

    async fn create_user<'a>(&self, user: &'a User) -> Result<User, Error> {
        let password = bcrypt::hash(&user.password, 12).unwrap();
        let query = self
            .client
            .query_one(
                "INSERT INTO users (id, name, email, password, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
                &[&user.id, &user.name, &user.email, &password, &user.created_at.unwrap(), &user.updated_at.unwrap()],
            )
            .await?;
        let role = Role::User;
        self.save_user_role(&user.clone().id.unwrap(), &role)
            .await?;
        let user = User {
            id: query.get(0),
            name: query.get(1),
            email: query.get(2),
            password: query.get(3),
            created_at: Some(query.get(4)),
            updated_at: Some(query.get(5)),
        };
        Ok(user)
    }

    async fn update_user_name<'a>(
        &self,
        user_name: &'a str,
        user_uid: &'a str,
    ) -> Result<User, Error> {
        let query = self
            .client
            .query_one(
                "UPDATE users SET name = $1 WHERE id = $2 RETURNING *",
                &[&user_name, &user_uid],
            )
            .await?;
        let user = User {
            id: query.get(0),
            name: query.get(1),
            email: query.get(2),
            password: query.get(3),
            created_at: Some(query.get(4)),
            updated_at: Some(query.get(5)),
        };
        Ok(user)
    }

    async fn get_user<'a>(&self, user_uid: &'a str) -> Result<User, Error> {
        let query = self
            .client
            .query_one("SELECT * FROM users WHERE id = $1", &[&user_uid])
            .await?;
        let user = User {
            id: query.get(0),
            name: query.get(1),
            email: query.get(2),
            password: query.get(3),
            created_at: Some(query.get(4)),
            updated_at: Some(query.get(5)),
        };
        Ok(user)
    }

    #[cfg(test)]
    async fn crate_random_user<'a>(&self) -> Result<User, Error> {
        use chrono::Utc;

        let random_string = uuid::Uuid::new_v4().to_string();
        let email = format!("{}@gmail.com", random_string);
        let user = User {
            id: Some(random_string),
            name: "test".to_string(),
            email,
            password: "password".to_string(),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        self.create_user(&user).await
    }

    #[cfg(test)]
    async fn create_test_user<'a>(&self, uuid: &'a str) -> Result<User, Error> {
        use chrono::Utc;
        let email = format!("{}@gmail.com", uuid);
        let user = User {
            id: Some(uuid.to_string()),
            name: "test".to_string(),
            email,
            password: "password".to_string(),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        self.create_user(&user).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::role::Role;
    use crate::structs::user::User;
    use crate::traits::user::UserTrait;
    use chrono::Utc;

    #[tokio::test]
    async fn test_create_user() {
        let mut _client = PostGreClient::new().await;
        let _db_dropeed = _client.drop_tables().await.unwrap();
        let _db_created = _client.create_tables_if_not_exist().await.unwrap();
        let random_string = uuid::Uuid::new_v4().to_string();
        let email = format!("{}@gmail.com", random_string);
        let user = User {
            id: Some(random_string),
            name: "test".to_string(),
            email: email.clone(),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            password: "password".to_string(),
        };
        let user_created = _client.create_user(&user).await.unwrap();
        assert_eq!(user.name, user.name);
        assert_eq!(user.email, user.email);
        assert_eq!(
            user.created_at.unwrap().timestamp(),
            user_created.created_at.unwrap().timestamp()
        );
        assert_eq!(
            user.updated_at.unwrap().timestamp(),
            user_created.updated_at.unwrap().timestamp()
        );
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut _client = PostGreClient::new().await;
        let _db_dropeed = _client.drop_tables().await.unwrap();
        let _db_created = _client.create_tables_if_not_exist().await.unwrap();
        let user = _client.crate_random_user().await.unwrap();
        let user_getted = _client.get_user(&user.id.unwrap()).await.unwrap();
        assert_eq!(user.name, user_getted.name);
        assert_eq!(user.email, user_getted.email);
        assert_eq!(
            user.created_at.unwrap().timestamp(),
            user_getted.created_at.unwrap().timestamp()
        );
        assert_eq!(
            user.updated_at.unwrap().timestamp(),
            user_getted.updated_at.unwrap().timestamp()
        );
    }

    #[tokio::test]
    async fn test_get_user_roles() {
        let mut _client = PostGreClient::new().await;
        let _db_dropeed = _client.drop_tables().await.unwrap();
        let _db_created = _client.create_tables_if_not_exist().await.unwrap();
        let user = _client.crate_random_user().await.unwrap();
        let roles = _client.get_user_roles(&user.id.unwrap()).await.unwrap();
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0], Role::User);
    }

    #[tokio::test]
    async fn test_update_user_name() {
        let mut _client = PostGreClient::new().await;
        let _db_dropeed = _client.drop_tables().await.unwrap();
        let _db_created = _client.create_tables_if_not_exist().await.unwrap();
        let user = _client.crate_random_user().await.unwrap();
        let user = _client
            .update_user_name(&"new name", &user.id.unwrap())
            .await
            .unwrap();
        assert_eq!(user.name, "new name");
    }
}
