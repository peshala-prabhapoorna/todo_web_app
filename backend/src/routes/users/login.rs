use axum::{extract::State, http::StatusCode, Json};
use sea_orm::{DatabaseConnection, IntoActiveModel, Set};

use crate::{
    queries::user_queries::{find_by_username, save_active_user},
    routes::users::ResponseUser,
    utilities::{
        app_error::AppError, hash::verify_password, jwt::create_token, token_wrapper::TokenWrapper,
    },
};

use super::{RequestCreateUser, ResponseDataUser};

pub async fn login(
    State(db): State<DatabaseConnection>,
    State(token_secret): State<TokenWrapper>,
    Json(request_user): Json<RequestCreateUser>,
) -> Result<Json<ResponseDataUser>, AppError> {
    let user = find_by_username(&db, request_user.username).await?;

    if !verify_password(&request_user.password, &user.password)? {
        return Err(AppError::new(
            StatusCode::UNAUTHORIZED,
            "incorrect username and/or password",
        ));
    }

    let token = create_token(&*token_secret, user.username.clone())?;

    let mut user = user.into_active_model();
    user.token = Set(Some(token));
    let user = save_active_user(&db, user).await?;

    let response = ResponseUser {
        id: user.id,
        username: user.username,
        token: user.token.unwrap(),
    };

    Ok(Json(ResponseDataUser { data: response }))
}
