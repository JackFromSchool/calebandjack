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

use shuttle_runtime::SecretStore;

use sqlx::postgres::PgPoolOptions;

use askama::Template;

#[shuttle_runtime::main]
pub async fn main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&format!("postgresql://postgres.vjjgvrjgtpidryehdqnh:{}@aws-0-us-west-1.pooler.supabase.com:6543/postgres", secrets.get("DB_PASSWORD").expect("Secret not found")).to_string())
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
        .route("/new_user", get(routes::new_user).post(apiroutes::new_user))
        .route("/new_user/username", post(apiroutes::new_username_validation))
        .layer(Extension(db));
    
    Ok(app.into())
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
