use axum::{
    Extension,
    response::Redirect,
    response::IntoResponse,
    response::Response,
    response::Html,
    extract::Form
};

use axum_extra::extract::{
    CookieJar,
    cookie::Cookie,
};

use axum_macros::debug_handler;

use sqlx::{
    PgPool,
    Row
};

use serde::Deserialize;

use cookie::time::Duration;

use askama::Template;

use crate::HtmlTemplate;

#[derive(Deserialize)]
pub struct Login {
    pub email: String,
    pub password: String,
}

#[derive(Debug)]
#[derive(sqlx::FromRow)]
struct User { 
    user_id: i32, 
    username: String, 
    email: String, 
    password: String, 
}

#[derive(sqlx::FromRow)]
struct Session {
    session_id: i32,
    user_id: i32,
    issued: sqlx::types::time::OffsetDateTime,
}

pub async fn login(
    jar: CookieJar,
    db: Extension<PgPool>,
    Form(form): Form<Login>,
) -> (CookieJar, impl IntoResponse) {
    
    sqlx::query("DELETE FROM sessions
WHERE age(issued, current_timestamp) > INTERVAL '1 day';")
        .execute(&*db)
        .await
        .unwrap();
    
    let correct_password = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1 AND password = $2")
        .bind(form.email.to_lowercase())
        .bind(form.password)
        .fetch_one(&*db)
        .await;

    match correct_password {
        Ok(user) => {
            sqlx::query("INSERT INTO sessions (user_id) VALUES ($1)")
                .bind(user.user_id)
                .execute(&*db)
                .await
                .unwrap();

            let new_session = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE user_id = $1")
                .bind(user.user_id)
                .fetch_one(&*db)
                .await
                .unwrap();

            let mut cookie = Cookie::new("session_id", new_session.session_id.to_string());
            cookie.set_max_age(Duration::hours(24));

            (jar.add(cookie), Redirect::to("/home"))
        },
        Err(_) => {
            (jar, Redirect::to("/login"))
        }
    }
}

pub async fn logout(
    jar: CookieJar,
    db: Extension<PgPool>,
) -> (CookieJar, impl IntoResponse) {
    let session_id = jar.get("session_id")
        .unwrap()
        .value_trimmed()
        .parse::<i32>()
        .unwrap();

    sqlx::query("DELETE FROM sessions WHERE session_id = $1")
        .bind(session_id)
        .execute(&*db)
        .await
        .unwrap();


    (jar.remove("session_id"), Redirect::to("/"))
}

#[derive(Deserialize)]
pub struct NewRecommendation {
    to: String,
    r_type: String,
    name: String,
    artist: String,
}

#[derive(Template)]
#[template(path = "new_success.html")]
struct NewSuccessTemplate {
    user: String,
}

pub async fn new(
    jar: CookieJar,
    Extension(db): Extension<PgPool>,
    Form(form): Form<NewRecommendation>
) -> impl IntoResponse {
    let for_id: i32 = sqlx::query("SELECT user_id FROM users WHERE username = $1")
        .bind(&form.to)
        .fetch_one(&db)
        .await
        .unwrap()
        .get_unchecked("user_id");
    
    let session_id = jar.get("session_id")
        .unwrap()
        .value_trimmed()
        .parse::<i32>()
        .unwrap();

    let from_id: i32 = sqlx::query("SELECT user_id FROM sessions WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&db)
        .await
        .unwrap()
        .get_unchecked("user_id");
    
    sqlx::query("INSERT INTO recommendations (name, type, artist, for_id, from_id)
VALUES ($1, $2, $3, $4, $5);")
        .bind(&form.name)
        .bind(&form.r_type)
        .bind(&form.artist)
        .bind(for_id)
        .bind(from_id)
        .execute(&db)
        .await
        .unwrap();

    let template = NewSuccessTemplate {
        user: form.to,
    };

    return template;
}

#[derive(Deserialize)]
pub struct UsernameValidation {
    to: String,
}

#[derive(Template)]
#[template(path = "username_validation.html")]
struct UsernameValidationTemplate<'a> {
    message: &'a str,
    class: &'a str,
    value: String,
}

pub async fn username_validation(
    Extension(db): Extension<PgPool>,
    Form(form): Form<UsernameValidation>,
) -> impl IntoResponse {
    if sqlx::query("SELECT * FROM users WHERE username = $1")
        .bind(&form.to.to_lowercase())
        .fetch_optional(&db)
        .await
        .unwrap()
        .is_some() {
        let template = UsernameValidationTemplate {
            message: "Username exists.",
            class: "valid",
            value: form.to.to_lowercase(),
        };

        return template;
    } else {
        let template = UsernameValidationTemplate {
            message: "Username does not exist.",
            class: "error",
            value: form.to.to_lowercase(),
        };

        return template;
    }
}

#[derive(Deserialize)]
pub struct Rating {
    rating: i16,
    comments: String,
    recommendation_id: i32,
}

pub async fn rate(
    Extension(db): Extension<PgPool>,
    Form(form): Form<Rating>,
) -> Response {
    sqlx::query("INSERT INTO reviews (rating, comments, recommendation_id)
VALUES ($1, $2, $3);")
        .bind(form.rating)
        .bind(form.comments)
        .bind(form.recommendation_id)
        .execute(&db)
        .await
        .unwrap();

    Html(r#"<p class="sent">Review sent!</p>
<a href="/home">Click to return home.</a>"#).into_response()
}
