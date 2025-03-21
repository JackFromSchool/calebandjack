use axum::{
    Extension,
    response::Redirect,
    response::Response,
    response::IntoResponse,
    response::Html,
    extract::Form
};

use axum_extra::extract::{
    CookieJar,
    cookie::Cookie,
};

use sqlx::{
    PgPool,
    Row
};

use serde::Deserialize;

use cookie::time::Duration;

use maud::html;

use crate::templates::username_validation_template;

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
        Err(err) => {
            println!("{}", err);
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
    valid_username: String,
}

pub async fn new(
    jar: CookieJar,
    Extension(db): Extension<PgPool>,
    Form(form): Form<NewRecommendation>
) -> Response {
    if form.valid_username == "false" {
        return html! {
            form hx-post="/new" hx-swap="outerHTML" hx-target="this" {
                div hx-target="this" hx-swap="outerHTML" class="form-div" {
                    label { "Send Recommendation To" }
                    input required name="to" value=(form.to) type="text" placeholder="Username" hx-post="/new/usernmae";
                    input type="hidden" name="valid_username" value="false";
                }
                label { "Type" }
                select required value=(form.r_type) name="r_type" {
                    option value="Album" { "Album" }
                    option value="Song" { "Song" }
                }
                label { "Name" }
                input required name="name" value=(form.name) type="text" placeholder="Name";
                label { "Artist" }
                input required name="artist" value=(form.artist) type="text" placeholder="Artist";
                input type="submit";
            }
        }.into_response();
    }
    
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

    html! {
        form hx-post="/new" hx-swap="outerHTML" hx-target="this" {
            div .form-div hx-target="this" hx-swap="outerHTML" {
                label { "Send Recommendation To" }
                input required name="to" type="text" placeholder="Username" hx-post="/new/username";
                input type="hidden" name="valid_username" value="false";
            }
            label { "Type" }
            select required name="r_type" {
                option value="Album" { "Album" }
                option value="Song" { "Song" }
            }
            label { "Name" }
            input required name="name" type="text" placeholder="Name";
            label { "Artist" }
            input required name="artist" type="text" placeholder="Artist";
            input type="submit";
        }
        p .valid { "Recommendation sent to " (form.to) "!"}
    }.into_response()
}

#[derive(Deserialize)]
pub struct UsernameValidation {
    to: String,
}

pub async fn username_validation(
    Extension(db): Extension<PgPool>,
    Form(form): Form<UsernameValidation>,
) -> impl IntoResponse {
    if form.to == "calebandjackhavingsex.com" {
        return username_validation_template(
            "What a wild and wacky name! DONT USE IT!",
            "error",
            form.to.to_lowercase(),
            "false"
        );
    }
    
    if sqlx::query("SELECT * FROM users WHERE username = $1")
        .bind(&form.to.to_lowercase())
        .fetch_optional(&db)
        .await
        .unwrap()
        .is_some() {

        return username_validation_template(
            "Username exists.",
            "valid",
            form.to.to_lowercase(),
            "true"
        );
    } else {
        return username_validation_template(
            "Username does not exist.",
            "error",
            form.to.to_lowercase(),
            "false"
        );
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

#[derive(Deserialize)]
pub struct NewUsernameValidation {
    username: String,
}

pub async fn new_username_validation(
    Extension(db): Extension<PgPool>,
    Form(form): Form<NewUsernameValidation>,
) -> Response {
    println!("Called new_username_validation api route");
    if sqlx::query("SELECT * FROM users WHERE username = $1")
        .bind(&form.username.to_lowercase())
        .fetch_optional(&db)
        .await
        .expect("Failed sqlx query")
        .is_none() && form.username.to_lowercase() != "calebandjackhavingsex.com" {

        return html! {
            div hx-target="this" hx-swap="outerHTML" class="form-div" {
                label { "Username" }
                input required name="username" type="text" value=(form.username.to_lowercase()) hx-post="/new_user/username" placeholder="Username";
                input type="hidden" name="valid_username" value="true";
                p .valid { "Username is available!"}
            }
        }.into_response();
    } else {

        return html! {
            div hx-target="this" hx-swap="outerHTML" class="form-div" {
                label { "Username" }
                input required name="username" type="text" value=(form.username.to_lowercase()) hx-post="/new_user/username" placeholder="Username";
                input type="hidden" name="valid_username" value="false";
                p .error { "Username already exists."}
            }
        }.into_response();
    }
}

#[derive(Deserialize)]
pub struct NewUser {
    username: String,
    email: String,
    password: String,
    valid_username: String,
}

pub async fn new_user(
    Extension(db): Extension<PgPool>,
    Form(form): Form<NewUser>,
) -> Response {
    if form.valid_username == "false" {
        return html! {
            form hx-post="/new_user" hx-swap="outerHTML" hx-target="this" {
                div hx-target="this" hx-swap="outerHTML" class="form-div" {
                    label { "Username" }
                    input required name="username" value=(form.username) type="text" hx-post="/new_user/username" placeholder="Username";
                    input type="hidden" name="valid_username" value="false";
                }
                label { "Email" }
                input required name="email" value=(form.email) type="email" placeholder="Email";
                label { "Password" }
                input required name="password" value=(form.password) type="text" placeholder="Password";
                input type="submit";
            }
        }.into_response();
    }
    
    sqlx::query("INSERT INTO users (username, email, password)
VALUES ($1, $2, $3);")
        .bind(form.username.to_lowercase())
        .bind(form.email)
        .bind(form.password)
        .execute(&db)
        .await
        .unwrap();

   return Html(r#"<div>
      <p class="valid">Account Created!</p>
      <a href="/login">Click here to login.</a>
   </div>"#).into_response();
}
