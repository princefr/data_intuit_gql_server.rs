mod contexts;
mod database;
mod enums;
mod firebase;
mod guards;
mod mutations;
mod queries;
mod structs;
mod traits;
mod utils;

use std::sync::Arc;

use database::main::PostGreClient;
use firebase::main::Firebase;
use mutations::main::Mutation;
use queries::main::Query;
use reqwest::Method;
use serde::Deserialize;
use structs::user::User;
use tokio::sync::{Mutex, RwLock};

use contexts::{token::Token, user_uid::UserUID};

use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig, ALL_WEBSOCKET_PROTOCOLS},
    EmptySubscription, Schema,
};
use async_graphql_poem::{GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLWebSocket};

use poem::{
    get, handler,
    http::HeaderMap,
    listener::TcpListener,
    middleware::Cors,
    web::{websocket::WebSocket, Data, Html},
    EndpointExt, IntoResponse, Route, Server,
};

// App Schema
pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

pub async fn on_connection_init(
    value: serde_json::Value,
) -> async_graphql::Result<async_graphql::Data> {
    #[derive(Deserialize)]
    struct Payload {
        token: String,
    }

    // Coerce the connection params into our `Payload` struct so we can
    // validate the token exists in the headers.
    if let Ok(payload) = serde_json::from_value::<Payload>(value) {
        let mut data = async_graphql::Data::default();
        data.insert(Token(payload.token));
        Ok(data)
    } else {
        Err("Token is required".into())
    }
}

/*

    This function is used to extract the token from the headers
    and pass it to the graphql context

*/

fn get_token_from_headers(headers: &HeaderMap) -> Option<Token> {
    let auth_header = headers.get("Authorization")?;
    let auth_header = auth_header.to_str().ok()?;
    let auth_header = auth_header.split(" ").collect::<Vec<_>>();

    let token = match auth_header.len() {
        0 => None,
        1 => None,
        _ => Some(auth_header[1].to_string()),
    };

    if token.is_none() {
        None
    } else {
        Some(Token(token.unwrap()))
    }
}

#[handler]
async fn graphiql() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/")))
}

#[handler]
async fn index(
    schema: Data<&AppSchema>,
    headers: &HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.0;
    if let Some(token) = get_token_from_headers(headers) {
        req = req.data(token);
    }

    schema.execute(req).await.into()
}

#[handler]
async fn ws(
    schema: Data<&AppSchema>,
    protocol: GraphQLProtocol,
    websocket: WebSocket,
) -> impl IntoResponse {
    let schema = schema.0.clone();
    websocket
        .protocols(ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |stream| {
            GraphQLWebSocket::new(stream, schema, protocol)
                // connection params are used to extract the token in this fn
                .on_connection_init(on_connection_init)
                .serve()
        })
}

pub async fn launch_server() -> Result<(), std::io::Error> {
    dotenv::dotenv().ok();
    // database for graphql consumption
    let database = PostGreClient::new().await;
    let database_arc_rw = Arc::new(RwLock::new(database));

    database_arc_rw
        .write()
        .await
        .drop_tables()
        .await
        .expect("Error dropping tables");
    let create_tables = database_arc_rw
        .write()
        .await
        .create_tables_if_not_exist()
        .await;
    match create_tables {
        Ok(_) => (),
        Err(e) => println!("Error creating tables: {:?}", e),
    }

    let firebase = Firebase::new().await;

    let user_iud: Arc<Mutex<UserUID>> = Arc::new(Mutex::new(UserUID("".to_string())));
    let user: Arc<Mutex<Option<User>>> = Arc::new(Mutex::new(None));

    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .data(database_arc_rw)
        .data(firebase)
        .data(user_iud)
        .data(user)
        .finish();

    let cors = Cors::new()
        .allow_method(Method::GET)
        .allow_method(Method::POST)
        .allow_method(Method::OPTIONS)
        .allow_origin("http://localhost:5173")
        .allow_credentials(false);

    let app = Route::new().at("/", get(graphiql).post(index)).data(schema);

    println!("server started on localhost:4000");

    Server::new(TcpListener::bind("127.0.0.1:4000"))
        .run(app)
        .await
        .unwrap();

    Ok(())
}
