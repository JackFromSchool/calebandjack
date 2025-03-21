use axum::{
    response::{
        IntoResponse,
        Response,
        Redirect,
    },
    Extension,
    extract::Query,
};

use sqlx::{
    PgPool,
    Row
};

use maud::{ Markup, html};

use serde::Deserialize;

use axum_extra::extract::CookieJar;


use crate::templates::{ base, Css };

pub async fn root() -> Markup {
    let stylesheet = Css(include_str!("../styles/index.css"));
    
    base(
        html! {
            link rel="preconnect" href="https://fonts.googleapis.com";
            link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
            link href="https://fonts.googleapis.com/css2?family=Roboto:ital,wght@0,100;0,300;0,400;0,500;0,700;0,900;1,100;1,300;1,400;1,500;1,700;1,900&family=Rubik+Mono+One&display=swap" rel="stylesheet";
        },
        html! {
            h1 { "calebandjack.com" }
            p { "The premier destination for music sharing." }
            div class="center" {
                a href="/login" { "Login" }
            }
            (stylesheet)   
        }
    )
}

pub async fn login(
    jar: CookieJar,
) -> Response {
    if jar.get("session_id").is_some() {
        return Redirect::to("/home").into_response();
    }

    let stylesheet = Css(include_str!("../styles/login.css"));
    
    base(
        html! {
            link rel="preconnect" href="https://fonts.googleapis.com";
            link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
            link href="https://fonts.googleapis.com/css2?family=Montserrat:ital,wght@0,100..900;1,100..900&family=Roboto:ital,wght@0,100;0,300;0,400;0,500;0,700;0,900;1,100;1,300;1,400;1,500;1,700;1,900&display=swap" rel="stylesheet";
        },
        html! {
            h1 { "Login" }
            div class="center" {
                form action="/login" method="post" {
                    label { "Email" }
                    input required name="email" type="email" placeholder="Email";
                    label { "Password" }
                    input required name="email" type="email" placeholder="Password";
                    input type="submit";
                }
            }
            div class="center" {
                a href="/new_user" { "Click here to make a new account." }
            }
            (stylesheet)
        }
    ).into_response()
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
    by: String,
    returned_on: sqlx::types::time::Date,
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
WHERE for_id = $1 
AND recommendation_id NOT IN (SELECT recommendation_id FROM reviews);")
        .bind(session_user_id)
        .fetch_all(&*db)
        .await
        .expect("Database connection failed.");

    let reviews = sqlx::query_as::<_, Review>("SELECT review_id, name, type, artist, username, returned_on
FROM reviews
INNER JOIN recommendations ON reviews.recommendation_id = recommendations.recommendation_id
INNER JOIN users ON recommendations.for_id = users.user_id
WHERE for_id = $1 OR from_id = $1;")
        .bind(session_user_id)
        .fetch_all(&*db)
        .await
        .expect("Database connection failed.");

    let stylesheet = Css(include_str!("../styles/home.css"));

    base(
        html! {
            link rel="preconnect" href="https://fonts.googleapis.com";
            link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
            link href="https://fonts.googleapis.com/css2?family=Montserrat:ital,wght@0,100..900;1,100..900&family=Roboto:ital,wght@0,100;0,300;0,400;0,500;0,700;0,900;1,100;1,300;1,400;1,500;1,700;1,900&display=swap" rel="stylesheet";
        },
        html! {
            h1 { 
                "Welcome back,"
                br;
                ( name )
            }
            .center {
                .shelf {
                    h2 { "Recommendations" }
                    hr;
                    @for recommendation in recommendations {
                        .list-container {
                            span {
                                h3 { (recommendation.name) p .type { (recommendation.r_type ) } }
                                p { (recommendation.artist)}
                                p { "From " (recommendation.from) " on " (recommendation.recommended_on) }
                            }
                            span .center-vertical {
                                a href={"recommendations?q=" (recommendation.id)} { "Rate" }
                            }
                        }
                    }
                }
            }
            .center {
                .shelf {
                    h2 { "Reviews" }
                    hr;
                    @for review in reviews {
                        .list-container {
                            span {
                                h3 { (review.name) p .type { (review.r_type ) } }
                                p { (review.artist)}
                                p { "From " (review.by) " on " (review.returned_on) }
                            }
                            span .center-vertical {
                                a href={"reviews?q=" (review.id)} { "View" }
                            }
                        }
                    }
                }
            }
            .center {
                .shelf {
                    h2 { "Send Recommendation" }
                    hr;
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
                }
            }
            .center {
                a href="/logout" { "Logout" }
            }
            (stylesheet)
        }
    ).into_response()
}


#[derive(Deserialize)]
pub struct OneQuery {
    q: i32,
}

pub async fn recommendations(
    jar: CookieJar,
    Extension(db): Extension<PgPool>,
    query: Query<OneQuery>,
) -> Response {
    let for_id: i32 = sqlx::query("SELECT for_id FROM recommendations
WHERE recommendation_id = $1;")
        .bind(&query.q)
        .fetch_one(&db)
        .await
        .unwrap()
        .get_unchecked("for_id");

    let session_id = jar.get("session_id").unwrap().value_trimmed().parse::<i32>().unwrap();
    let user_id: i32 = sqlx::query("SELECT user_id FROM sessions WHERE session_id = $1;")
        .bind(session_id)
        .fetch_one(&db)
        .await
        .unwrap()
        .get_unchecked("user_id");

    if user_id != for_id {

        let stylesheet = Css(include_str!("../styles/protected.css"));

        return base(
            html! {
                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
                link href="https://fonts.googleapis.com/css2?family=Montserrat:ital,wght@0,100..900;1,100..900&family=Roboto:ital,wght@0,100;0,300;0,400;0,500;0,700;0,900;1,100;1,300;1,400;1,500;1,700;1,900&display=swap" rel="stylesheet";
            },
            html! {
                h1 { "This page is protected" }
                p { "This recommendation is not for you" }
                .center {
                    a href="/home" { "Click here to go home" }
                }
                (stylesheet)
            }
        ).into_response();
    }
    
    let recommendation = sqlx::query_as::<_, Recommendation>("SELECT recommendation_id, name, type, artist, recommended_on, username
FROM recommendations
INNER JOIN users ON recommendations.from_id = users.user_id
WHERE recommendation_id = $1;")
        .bind(&query.q)
        .fetch_one(&db)
        .await
        .unwrap();

    let stylesheet = Css(include_str!("../styles/recommendation.css"));
    
    base(
        html! {
            link rel="preconnect" href="https://fonts.googleapis.com";
            link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
            link href="https://fonts.googleapis.com/css2?family=Montserrat:ital,wght@0,100..900;1,100..900&family=Roboto:ital,wght@0,100;0,300;0,400;0,500;0,700;0,900;1,100;1,300;1,400;1,500;1,700;1,900&display=swap" rel="stylesheet";
        },
        html! {
            h1 { (recommendation.name) }
            p .subtitle {
                "A"
                @if recommendation.r_type == "Album" { "n" }
                " "
                (recommendation.r_type)
                ", by "
                (recommendation.artist)
            }
            p .subtitle style="margin-bottom: 2rem;" {
                "Recommended to you by "
                (recommendation.from)
            }

            .center {
                .shelf {
                    h2 { "Rate" }
                    hr;
                    form hx-post="/rate" hx-swap="outerHTML" hx-target="this" {
                        label { "Rating (Out of Ten)"}
                        span .rating-container {
                            input required name="rating" id="input" type="range" min="0" max="10" value ="0";
                            p #display { "num" }
                        }
                        label { "Comments" }
                        textarea name="comments" rows="5" placeholder="Type here or leave blank" {}
                        input type="hidden" name="recommendation_id" value={ (recommendation.id) };
                        input type="submit";
                    }
                }
            }
            script {
                r##"
const input = document.querySelector("#input");
const display = document.querySelector("#display");

display.textContent = input.value;
input.addEventListener("input", (event) => {
    display.textContent = event.target.value;
})
                "##
            }
            (stylesheet)
        }
    ).into_response()
}

#[derive(sqlx::FromRow)]
struct ReviewOwners {
    for_id: i32,
    from_id: i32,
}

#[derive(sqlx::FromRow)]
struct FullReview {
    name: String,
    r_type: String,
    artist: String,
    rating: i16,
    comments: String,
    by: String,
    returned_on: sqlx::types::time::Date,
    from: String,
    recommended_on: sqlx::types::time::Date,
}

pub async fn reviews(
    jar: CookieJar,
    Extension(db): Extension<PgPool>,
    query: Query<OneQuery>,
) -> Response {
    let owners = sqlx::query_as::<_, ReviewOwners>("SELECT for_id, from_id FROM reviews
INNER JOIN recommendations ON reviews.recommendation_id = recommendations.recommendation_id
WHERE review_id = $1;")
        .bind(&query.q)
        .fetch_one(&db)
        .await
        .unwrap();

    let session_id = jar.get("session_id").unwrap().value_trimmed().parse::<i32>().unwrap();
    let user_id: i32 = sqlx::query("SELECT user_id FROM sessions WHERE session_id = $1;")
        .bind(session_id)
        .fetch_one(&db)
        .await
        .unwrap()
        .get_unchecked("user_id");

    if user_id != owners.for_id && user_id != owners.from_id {
        let stylesheet = Css(include_str!("../styles/protected.css"));

        return base(
            html! {
                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
                link href="https://fonts.googleapis.com/css2?family=Montserrat:ital,wght@0,100..900;1,100..900&family=Roboto:ital,wght@0,100;0,300;0,400;0,500;0,700;0,900;1,100;1,300;1,400;1,500;1,700;1,900&display=swap" rel="stylesheet";
            },
            html! {
                h1 { "This page is protected" }
                p { "This review is not for you" }
                .center {
                    a href="/home" { "Click here to go home" }
                }
                (stylesheet)
            }
        ).into_response();
    }
    
    let full_review = sqlx::query_as::<_, FullReview>("SELECT name, type as r_type, artist, rating, comments, u1.username as by, returned_on, u2.username as from, recommended_on
FROM reviews 
INNER JOIN recommendations ON reviews.recommendation_id = recommendations.recommendation_id
INNER JOIN users u1 ON recommendations.for_id = u1.user_id
INNER JOIN users u2 ON recommendations.from_id = u2.user_id
WHERE review_id = $1;")
        .bind(&query.q)
        .fetch_one(&db)
        .await
        .unwrap();

    let stylesheet = Css(include_str!("../styles/review.css"));
    
    base(
        html! {
            link rel="preconnect" href="https://fonts.googleapis.com";
            link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
            link href="https://fonts.googleapis.com/css2?family=Montserrat:ital,wght@0,100..900;1,100..900&family=Roboto:ital,wght@0,100;0,300;0,400;0,500;0,700;0,900;1,100;1,300;1,400;1,500;1,700;1,900&display=swap" rel="stylesheet";
        },
        html! {
            h1 { (full_review.name) }
            p .subtitle {
                "A"
                @if full_review.r_type == "Album" { "n" }
                " "
                (full_review.r_type)
                ", by "
                (full_review.artist)
            }
            .center {
                .shelf {
                    h2 { "Review" }
                    hr;
                    h3 { (full_review.rating) "/10" }
                    p { (full_review.comments) }
                    br;
                    p .more-margin { "Reviewed by " (full_review.by) " on " (full_review.returned_on)}
                }
            }
            .center {
                .shelf {
                    h2 { "Recommendation" }
                    hr;
                    p .more-margin { "Recommended by " (full_review.from) " on " (full_review.recommended_on)}
                }
            }
            (stylesheet)
        }
    ).into_response()
}

pub async fn new_user() -> Response {
    let stylesheet = Css(include_str!("../styles/new_user.css"));
    
    base(
        html! {
            link rel="preconnect" href="https://fonts.googleapis.com";
            link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
            link href="https://fonts.googleapis.com/css2?family=Montserrat:ital,wght@0,100..900;1,100..900&family=Roboto:ital,wght@0,100;0,300;0,400;0,500;0,700;0,900;1,100;1,300;1,400;1,500;1,700;1,900&display=swap" rel="stylesheet";
        },
        html! {
            h1 { "New User" }
            .center {
                form hx-post="/new_user" hx-swap="outerHTML" hx-target="this" {
                    div hx-target="this" hx-swap="outerHTML" class="form-div" {
                        label { "Username" }
                        input required name="username" type="text" hx-post="/new_user/username" placeholder="Username";
                        input type="hidden" name="valid_username" value="false";
                    }
                    label { "Email" }
                    input required name="email" type="email" placeholder="Email";
                    label { "Password" }
                    input required name="password" type="text" placeholder="Password";
                    input type="submit";
                }
                (stylesheet)
            }
        }
    ).into_response()
}
