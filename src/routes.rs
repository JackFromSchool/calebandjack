use axum::{
    response::{
        Response,
        IntoResponse,
        Redirect,
    },
    Extension,
};

use sqlx::{
    PgPool,
    Row
};

use axum_extra::extract::CookieJar;

use askama_axum::Template;

use crate::HtmlTemplate;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

pub async fn root() -> impl IntoResponse {
    let template = IndexTemplate;
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

pub async fn login(
    jar: CookieJar,
) -> Response {
    if jar.get("session_id").is_some() {
        return Redirect::to("/home").into_response();
    }
    
    let template = LoginTemplate;
    HtmlTemplate(template).into_response()
}

#[derive(sqlx::FromRow)]
pub struct Review {
    #[sqlx(rename = "review_id")]
    id: i32,
    name: String,
    #[sqlx(rename = "type")]
    r_type: String,
    artist: String,
    #[sqlx(rename = "username")]
    from: String,
    recommended_on: sqlx::types::time::Date,
}

#[derive(sqlx::FromRow)]
pub struct Recommendation {
    #[sqlx(rename = "recommendation_id")]
    id: i32,
    name: String,
    #[sqlx(rename = "type")]
    r_type: String,
    artist: String,
    #[sqlx(rename = "username")]
    from: String,
    recommended_on: sqlx::types::time::Date,
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    name: String,
    reviews: Vec<Review>,
    recommendations: Vec<Recommendation>,
}

pub async fn home(
    jar: CookieJar,   
    db: Extension<PgPool>,
) -> Response {
    let session_user_id: i32;
    
    match jar.get("session_id") {
        Some(session_id) => {
            if sqlx::query("SELECT * FROM sessions WHERE session_id = $1")
                .bind(session_id.value_trimmed().parse::<i32>().unwrap())
                .fetch_optional(&*db)
                .await
                .expect("Database connection failed.")
                .is_some() {
                let rows = sqlx::query("SELECT user_id FROM sessions WHERE session_id = $1")
                    .bind(session_id.value_trimmed().parse::<i32>().unwrap())
                    .fetch_one(&*db)
                    .await
                    .expect("Database connection failed.");

                session_user_id = rows.get_unchecked("user_id");
            } else {
                return Redirect::to("/login").into_response();
            }
        },
        None => {
            return Redirect::to("/login").into_response();
        }
    }

    let name: String = sqlx::query("SELECT username FROM users WHERE user_id = $1")
        .bind(session_user_id)
        .fetch_one(&*db)
        .await
        .expect("Database connection failed.")
        .get_unchecked("username");
    
    let recommendations = sqlx::query_as::<_, Recommendation>(
        "SELECT recommendation_id, name, type, artist, recommended_on, username
FROM recommendations
INNER JOIN users ON recommendations.from_id = users.user_id
WHERE for_id = $1;")
        .bind(session_user_id)
        .fetch_all(&*db)
        .await
        .expect("Database connection failed.");

    let reviews = sqlx::query_as::<_, Review>("SELECT review_id, name, type, artist, username, recommended_on
FROM reviews
INNER JOIN recommendations ON reviews.recommendation_id = recommendations.recommendation_id
INNER JOIN users ON recommendations.from_id = users.user_id
WHERE by_id = $1;")
        .bind(session_user_id)
        .fetch_all(&*db)
        .await
        .expect("Database connection failed.");

    let template = HomeTemplate {
        name,
        recommendations,
        reviews,
    };
    template.into_response()
}
