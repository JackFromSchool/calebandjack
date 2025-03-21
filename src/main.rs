mod routes;
mod apiroutes;
mod templates;

use axum::{
    Extension,
    routing::get,
    routing::post,
    Router,
};

use shuttle_shared_db;

#[shuttle_runtime::main]
pub async fn main(#[shuttle_shared_db::Postgres] pool: sqlx::PgPool
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .unwrap();
    
    let app = Router::new()
        .route("/", get(routes::root))
        .route("/login", get(routes::login).post(apiroutes::login))
        .route("/logout", get(apiroutes::logout))
        .route("/home", get(routes::home))
        .route("/new", post(apiroutes::new))
        .route("/new/username", post(apiroutes::username_validation))
        .route("/recommendations", get(routes::recommendations))
        .route("/rate", post(apiroutes::rate))
        .route("/reviews", get(routes::reviews))
        .route("/new_user", get(routes::new_user).post(apiroutes::new_user))
        .route("/new_user/username", post(apiroutes::new_username_validation))
        .layer(Extension(pool));
    
    Ok(app.into())
}
