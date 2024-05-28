[![Build](https://github.com/princefr/data_intuit_gql_server.rs/actions/workflows/deployment.yml/badge.svg)](https://github.com/princefr/data_intuit_gql_server.rs/actions/workflows/deployment.yml)


# Graphql Server.
# Test

To run the server and test the graphql queries, run the following commands you will first need to create a `.env` file in the root directory with the following content:

```env
POSTGRES_HOST=
POSTGRES_USER=
POSTGRES_PASSWORD=
POSTGRES_DATABASE=
POSTGRES_PORT=
FIREBASE_API_KEY=
SERVICE_ACCOUNT='service account json'
```


make sure to have SERVICE_ACCOUNT (Firebase Service Account json) json data into quotes preferably  => ''.

Tests can be run with the following command:

```bash
cargo test -- --test-threads=1
```


Query and Mutation are tested.
Database cruds are tested.
Firebase authentication is tested.


As this is just a poc to showcase how things can be done, i rushed through the code and didn't follow the best practices (not having unwraps, handle errors properly etc).

**I didn't add any logging, error handling, and proper response handling.**

Graphql QL playground can be accessed at `http://localhost:4000/` once the server is launched
