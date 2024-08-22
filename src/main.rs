mod routes;
mod apiroutes;

use axum::{
    Extension,
    http::StatusCode,
    routing::get,
    routing::post,
    Router,
    response::{Response, Html, IntoResponse},
};

use sqlx::postgres::PgPoolOptions;

use askama::Template;

#[tokio::main]
async fn main() {
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost:5432/postgres")
        .await
        .expect("Failed to connect to database.");
    
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
        .layer(Extension(db));

    let listener = tokio::net::TcpListener::bind("192.168.1.187:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
    where
        T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            ).into_response(),
        }
    }
}
