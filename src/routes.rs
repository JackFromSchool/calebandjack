use axum::{
    response::{
        Response,
        IntoResponse,
        Redirect,
    },
    Extension,
    extract::Query,
};

use sqlx::{
    PgPool,
    Row
};

use serde::Deserialize;

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

    let template = HomeTemplate {
        name,
        recommendations,
        reviews,
    };
    template.into_response()
}

#[derive(Template)]
#[template(path = "protected.html")]
struct ProtectedTemplate<'a> {
    message: &'a str,
}

#[derive(Deserialize)]
pub struct OneQuery {
    q: i32,
}

#[derive(Template)]
#[template(path = "recommendation.html")]
struct RecommendationTemplate {
    recommendation: Recommendation,
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
        let template = ProtectedTemplate {
            message: "This recommendation is not for you."
        };

        return template.into_response();
    }
    
    let recommendation = sqlx::query_as::<_, Recommendation>("SELECT recommendation_id, name, type, artist, recommended_on, username
FROM recommendations
INNER JOIN users ON recommendations.from_id = users.user_id
WHERE recommendation_id = $1;")
        .bind(&query.q)
        .fetch_one(&db)
        .await
        .unwrap();
    
    let template = RecommendationTemplate {
        recommendation,
    };

    template.into_response()
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

#[derive(Template)]
#[template(path = "review.html")]
struct ReviewTemplate {
    review: FullReview,
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
        let template = ProtectedTemplate {
            message: "This review is not for you."
        };

        return template.into_response();
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
        
    let template = ReviewTemplate {
        review: full_review,
    };

    return template.into_response();
}
