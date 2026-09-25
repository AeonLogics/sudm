use dotenv::var;
use surrealdb::engine::any::{connect, Any};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

pub async fn connection() -> Surreal<Any> {
    let url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    let user = var("SURREAL_USER").expect("SURREAL_USER must be set");
    let pass = var("SURREAL_PASS").expect("SURREAL_PASS must be set");
    let ns = var("NAMESPACE").expect("NAMESPACE must be set");
    let db_name = var("DATABASE_NAME").expect("DATABASE_NAME must be set");

    if !url.starts_with("ws://")
        && !url.starts_with("wss://")
        && !url.starts_with("http://")
        && !url.starts_with("https://")
    {
        panic!(
            "\n[Configuration Error]: Invalid DATABASE_URL ('{}').\n\
            SurrealDB requires a explicit protocol engine prefix.\n\
            Please update your environment file to include a scheme like:\n\
            - ws://localhost:8000\n\
            - wss://cloud.surrealdb.com\n\
            - http://localhost:8000\n",
            url
        );
    }

    let database = connect(&url)
        .await
        .unwrap_or_else(|err| {
            panic!(
                "Failed to connect to SurrealDB endpoint at '{}'. \n\
                Verify the server is actively running and accessible.\n\
                Underlying driver error: {}",
                url, err
            )
        });

    database
        .signin(Root {
            username: user,
            password: pass,
        })
        .await
        .expect("Database authentication failed with the provided credentials");

    database
        .use_ns(&ns)
        .use_db(&db_name)
        .await
        .expect("Failed to bind to specified Namespace or Database scope");

    println!("Successfully connected and authenticated to SurrealDB!");
    database
}
