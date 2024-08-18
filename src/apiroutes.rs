use axum::{
    Extension,
    response::Redirect,
    response::IntoResponse,
    response::Response,
    extract::Form
};

use axum_extra::extract::{
    CookieJar,
    cookie::Cookie,
};

use sqlx::PgPool;

use serde::Deserialize;

use cookie::time::Duration;

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
    issued: sqlx::types::time::PrimitiveDateTime,
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
        .bind(form.email)
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

#[derive(Deserialize)]
struct NewRecommendation {
    to: String,
    r_type: String,
    name: String,
    artist: String,
}

pub async fn new(
    Form(form): Form<NewRecommendation>
) -> Response {
    
}
