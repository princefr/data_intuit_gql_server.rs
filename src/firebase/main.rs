use crate::structs::user::User;
use async_graphql::*;
use gcp_auth::AuthenticationManager;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rs_firebase_admin_sdk::{
    auth::{
        token::TokenVerifier, AttributeOp, FirebaseAuthService, NewUser, UserIdentifiers,
        UserUpdate,
    },
    App, CustomServiceAccount, LiveAuthAdmin,
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
struct Payload {
    token: String,
    #[serde(rename = "returnSecureToken")]
    return_secure_token: bool,
}

#[derive(Deserialize)]
struct RespBody {
    #[serde(rename = "idToken")]
    id_token: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    iss: String,
    sub: String,
    aud: String,
    iat: i64,
    exp: i64,
    uid: String,
    claims: CustomClaims,
}

#[derive(Serialize, Deserialize)]
struct CustomClaims {
    premium_account: bool,
}

pub struct Firebase {
    app: App<AuthenticationManager>,
    verify_custom_url: String,
    service_account: CustomServiceAccount,
}

impl Firebase {
    /*
        * Create a new Firebase instance
        @return Firebase
    */
    pub async fn new() -> Firebase {
        dotenv::dotenv().ok();
        let key: String =
            env::var("SERVICE_ACCOUNT").expect("Firebase Service account key must be set");
        let service_account = CustomServiceAccount::from_json(&key).unwrap();
        let _service_account = CustomServiceAccount::from_json(&key).unwrap();
        let app = App::live(service_account.into()).await.unwrap();
        let api_key = env::var("FIREBASE_API_KEY").expect("Firebase API key must be set");
        Firebase {
            app,
            verify_custom_url: format!("https://www.googleapis.com/identitytoolkit/v3/relyingparty/verifyCustomToken?key={}", api_key),
            service_account:_service_account,
        }
    }

    /*
        * Create a user
        @param user: &UserInput
        @return Result<String, reqwest::Error>
    */
    pub async fn create_user(&self, user: &User) -> Result<String, reqwest::Error> {
        let client: LiveAuthAdmin = self.app.auth();
        let new_user = NewUser {
            email: Some(user.email.clone()),
            password: Some(user.password.clone()),
            uid: Some(user.id.clone().unwrap()),
        };
        let user = client
            .create_user(new_user)
            .await
            .expect("Error creating user");
        Ok(user.uid)
    }

    /*
        * Remove a user
        @param uid: &str
        @return Result<(), reqwest::Error>
    */
    pub async fn remove_user(&self, uid: &str) -> Result<(), reqwest::Error> {
        let client: LiveAuthAdmin = self.app.auth();
        client
            .delete_user(uid.to_string())
            .await
            .expect("Error deleting user");
        Ok(())
    }

    /*
        * Remove all users
        @return Result<(), reqwest::Error>
    */
    pub async fn delete_all_users(&self) -> Result<(), reqwest::Error> {
        let client: LiveAuthAdmin = self.app.auth();
        let users_list = client
            .list_users(1000, None)
            .await
            .expect("Error listing users");
        let users = users_list.unwrap().users;
        for user in users.iter() {
            client
                .delete_user(user.uid.clone())
                .await
                .expect("Error deleting user");
        }

        Ok(())
    }

    /*
        * Create a custom token for a user
        @param uid: &str
        @param is_premium_account: bool
        @return Result<String>
    */
    pub async fn create_custom_token(&self, uid: &str, is_premium_account: bool) -> Result<String> {
        let now_seconds = chrono::Utc::now().timestamp();
        let one_hour_from_now = now_seconds + 3600;

        let iss = format!(
            "{}.iam.gserviceaccount.com",
            "firebase-adminsdk-yc1ep@exchange-dev-afae1"
        );
        let sub = format!(
            "{}.iam.gserviceaccount.com",
            "firebase-adminsdk-yc1ep@exchange-dev-afae1"
        );

        let claims = Claims {
            iss,
            sub,
            aud: "https://identitytoolkit.googleapis.com/google.identity.identitytoolkit.v1.IdentityToolkit".to_string(),
            iat: now_seconds,
            exp: one_hour_from_now,  // Maximum expiration time is one hour
            uid: uid.to_string(),
            claims: CustomClaims {
                premium_account: is_premium_account,
            }
        };

        let key =
            EncodingKey::from_rsa_pem(self.service_account.private_key_pem().as_bytes()).unwrap();

        // Encode the token
        let token =
            encode(&Header::new(Algorithm::RS256), &claims, &key).expect("Failed to encode token");
        Ok(token)
    }

    /*

        * update user in firebase
        @param uid: &str
        @param user: &UserInput
        @return Result<(), reqwest::Error>
    */
    pub async fn update_user(&self, uid: &str, user: &User) -> Result<(), reqwest::Error> {
        let client: LiveAuthAdmin = self.app.auth();
        // let phone_formatted = format!("+33{}", user.clone().phone.replace(" ", ""));
        let display_name = AttributeOp::Change(user.name.clone());
        client
            .update_user(
                UserUpdate::builder(uid.to_string())
                    .email(user.email.clone())
                    .disabled(false)
                    .email_verified(false)
                    .password(user.password.clone())
                    .display_name(display_name)
                    .build(),
            )
            .await
            .expect("Failed to update user");
        Ok(())
    }

    /*
        * Get id token from custom token
        @param custom_token: custom token to verify
        @return: id token if custom token is valid, error otherwise
    */
    pub async fn get_id_token(&self, custom_token: &str) -> Result<String, reqwest::Error> {
        let payload = Payload {
            token: custom_token.to_string(),
            return_secure_token: true,
        };

        let client = reqwest::Client::new();
        let resp = client
            .post(&self.verify_custom_url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&payload).unwrap())
            .send()
            .await?
            .text()
            .await?
            .to_string();

        let resp_body: RespBody = serde_json::from_str(&resp).unwrap();

        Ok(resp_body.id_token)
    }

    /*
        * Verify id token
        @param id_token: id token to verify
        @return: user id if token is valid, error otherwise
    */
    pub async fn verify_id_token(&self, id_token: &str) -> Result<String> {
        let token_verifier = self.app.id_token_verifier().await.unwrap();
        match token_verifier.verify_token(id_token).await {
            Ok(token) => {
                let user_id = token.critical_claims.sub;
                return Ok(user_id);
            }
            Err(_) => return Err(Error::new("Unauthorized")),
        }
    }

    /*
        * Update email is verified
        @param uid: user id
        @return: true if email is verified, false otherwise
    */
    pub async fn update_email_is_verified(&self, uid: &str) -> Result<(), reqwest::Error> {
        let client: LiveAuthAdmin = self.app.auth();
        client
            .update_user(
                UserUpdate::builder(uid.to_string())
                    .email_verified(true)
                    .build(),
            )
            .await
            .expect("Failed to update user");
        Ok(())
    }

    /*
        * Check if password is valid
        @param uid: user id
        @param password: password to check
        @return: true if password is valid, false otherwise
    */
    pub async fn check_password(&self, uid: &str, password: &str) -> Result<bool, reqwest::Error> {
        let client: LiveAuthAdmin = self.app.auth();
        let user_identifier = UserIdentifiers::builder().with_uid(uid.to_string()).build();
        let user = client
            .get_user(user_identifier)
            .await
            .expect("Failed to create custom token");
        let user = user.unwrap();
        let user_password = user.password_hash;
        let is_valid =
            bcrypt::verify(password, &user_password.unwrap()).expect("Failed to verify password");
        Ok(is_valid)
    }

    /*
        * Change password
        @param uid: user id
        @param new_password: new password

        Change the password of a user
        @return: ()
        @throws: reqwest::Error
    */
    pub async fn change_password(
        &self,
        uid: &str,
        new_password: &str,
    ) -> Result<(), reqwest::Error> {
        let client: LiveAuthAdmin = self.app.auth();
        client
            .update_user(
                UserUpdate::builder(uid.to_string())
                    .password(new_password.to_string())
                    .build(),
            )
            .await
            .expect("Failed to update user");
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use chrono::Utc;
    use fake::Fake;
    // using `faker` module with locales
    use crate::structs::user::User;
    use fake::faker::internet::en::*;
    use fake::faker::name::raw::*;
    use fake::faker::number::raw::*;
    use fake::locales::*;

    use super::*;

    #[tokio::test]
    async fn test_create_custom_token() {
        let firebase = Firebase::new().await;
        let uid = "2323ZA2424test";
        let token = firebase.create_custom_token(uid, false);
        assert!(token.await.is_ok());
        //assert_eq!(token.await.unwrap(), "test")
    }

    #[tokio::test]
    async fn test_get_id_token() {
        let firebase = Firebase::new().await;
        let uid = "test";
        let token = firebase.create_custom_token(uid, false).await;
        assert!(token.is_ok());
        let res = firebase.get_id_token(&token.unwrap()).await;
        assert!(res.is_ok())
    }

    #[tokio::test]
    async fn test_verify_id_token() {
        let firebase = Firebase::new().await;
        let uid = "423423test";
        let token = firebase.create_custom_token(uid, false).await;
        let id_token_res = firebase.get_id_token(&token.unwrap()).await;
        assert!(id_token_res.is_ok());
        let res = firebase.verify_id_token(&id_token_res.unwrap()).await;
        assert!(res.is_ok())
    }

    #[tokio::test]
    async fn test_update_user() {
        let firebase = Firebase::new().await;
        let uid = Digit(EN).fake::<String>();
        let custom_token = firebase.create_custom_token(&uid, false).await;
        firebase
            .get_id_token(&custom_token.unwrap())
            .await
            .expect("Failed to get id token");

        let user = User {
            id: Some(uid.clone()),
            name: Name(EN).fake(),
            email: SafeEmail().fake(),
            password: "11794581oooooo&".to_string(),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let res = firebase.update_user(&uid, &user).await;
        assert!(res.is_ok())
    }

    #[tokio::test]
    async fn test_create_user() {
        let firebase = Firebase::new().await;

        let user = User {
            id: Some(Digit(EN).fake::<String>()),
            name: Name(EN).fake(),
            email: SafeEmail().fake(),
            password: "11794581oooooo&".to_string(),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let res = firebase.create_user(&user).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_remove_user() {
        let firebase = Firebase::new().await;

        let user = User {
            id: Some(Digit(EN).fake::<String>()),
            name: Name(EN).fake(),
            email: SafeEmail().fake(),
            password: "11794581oooooo&".to_string(),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        let res = firebase.create_user(&user).await;
        assert!(res.is_ok());
        let uid = res.unwrap();
        let res = firebase.remove_user(&uid).await;
        assert!(res.is_ok())
    }

    #[tokio::test]
    async fn test_delete_all_users() {
        let firebase = Firebase::new().await;
        let res = firebase.delete_all_users().await;
        assert!(res.is_ok())
    }
}
