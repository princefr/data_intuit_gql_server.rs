use std::env;
use std::sync::Arc;
use tokio_postgres::{Error, NoTls};

#[derive(Clone)]
pub struct PostGreClient {
    pub client: Arc<tokio_postgres::Client>,
}

impl PostGreClient {
    pub async fn new() -> PostGreClient {
        dotenv::dotenv().ok();
        let host = env::var("POSTGRES_HOST").expect("POSTGRES_HOST must be set");
        let user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set");
        let password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");
        let dbname = env::var("POSTGRES_DATABASE").expect("POSTGRES_DATABASE must be set");
        let port = env::var("POSTGRES_PORT").expect("POSTGRES_PORT must be set");

        let config = format!(
            "host={} user={} password={} dbname={}  port={}",
            host, user, password, dbname, port
        );
        let (client, connection) = tokio_postgres::connect(&config, NoTls).await.unwrap();

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        PostGreClient {
            client: Arc::new(client),
        }
    }

    /*
     * Create tables if not exist
     */
    pub async fn create_tables_if_not_exist(&mut self) -> Result<(), Error> {
        self.client
            .batch_execute(
                "CREATE TABLE IF NOT EXISTS users (
                    id TEXT PRIMARY KEY UNIQUE NOT NULL,
                    name TEXT NOT NULL,
                    email TEXT NOT NULL,
                    password TEXT NOT NULL,
                    created_at TIMESTAMPTZ NOT NULL default now(),
                    updated_at TIMESTAMPTZ NOT NULL default now()
                );",
            )
            .await?;

        self.client
            .batch_execute(
                "
            CREATE TYPE ROLE AS ENUM ('User', 'Manager', 'Admin');
            CREATE TABLE IF NOT EXISTS roles (
                id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
                role ROLE NOT NULL default 'User',
                firebase_uid TEXT not null references users(id) on delete cascade,
                created_at TIMESTAMPTZ NOT NULL default now(),
                updated_at TIMESTAMPTZ NOT NULL default now()
            );",
            )
            .await?;
        Ok(())
    }

    pub async fn drop_tables(&mut self) -> Result<(), Error> {
        self.client
            .batch_execute(
                "
                DROP TABLE IF EXISTS roles;
                DROP TABLE IF EXISTS users;
                DROP TYPE IF EXISTS ROLE;
                ",
            )
            .await?;
        Ok(())
    }
}
