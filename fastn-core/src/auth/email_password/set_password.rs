use crate::auth::email_password::{
    email_reset_request_sent_ftd, generate_key, redirect_url_from_next, set_password_form_ftd
};

/// GET | POST /-/auth/forgot-password/
/// POST forgot_password_request: send email with a link containing a key to reset password
/// for unauthenticated users
async fn forgot_password_request(
    req_config: &mut fastn_core::RequestConfig,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    if req_config.request.method() == "GET" {
        let main = fastn_core::Document {
            package_name: req_config.config.package.name.clone(),
            id: "/-/password-reset-request-sent".to_string(),
            content: email_reset_request_sent_ftd().to_string(),
            parent_path: fastn_ds::Path::new("/"),
        };

        let resp = fastn_core::package::package_doc::read_ftd(req_config, &main, "/", false, false)
            .await?;

        return Ok(resp.into());
    }

    if req_config.request.method() != "POST" {
        return Ok(fastn_core::not_found!("invalid route"));
    }

    #[derive(serde::Deserialize)]
    struct Payload {
        #[serde(rename = "username")]
        email_or_username: String,
    }

    let payload = req_config.request.json::<Payload>();

    if let Err(e) = payload {
        return fastn_core::http::user_err(
            vec![
                ("payload".into(), vec![format!("invalid payload: {:?}", e)]),
                (
                    "username".into(),
                    vec!["username/email is required".to_string()],
                ),
            ],
            fastn_core::http::StatusCode::OK,
        );
    }

    let payload = payload.unwrap();

    if payload.email_or_username.is_empty() {
        return fastn_core::http::user_err(
            vec![(
                "username".into(),
                vec!["username/email is required".to_string()],
            )],
            fastn_core::http::StatusCode::OK,
        );
    }

    let mut conn = db_pool
        .get()
        .await
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to get connection to db. {:?}", e),
        })?;

    let query = fastn_core::schema::fastn_user::table
        .inner_join(fastn_core::schema::fastn_user_email::table)
        .filter(fastn_core::schema::fastn_user::username.eq(&payload.email_or_username))
        .or_filter(
            fastn_core::schema::fastn_user_email::email
                .eq(fastn_core::utils::citext(&payload.email_or_username)),
        )
        .select((
            fastn_core::auth::FastnUser::as_select(),
            fastn_core::schema::fastn_user_email::email,
        ));

    dbg!("{:?}", diesel::debug_query::<diesel::pg::Pg, _>(&query));

    let user: Option<(fastn_core::auth::FastnUser, fastn_core::utils::CiString)> =
        query.first(&mut conn).await.optional()?;

    if user.is_none() {
        return fastn_core::http::user_err(
            vec![(
                "username".into(),
                vec!["invalid email/username".to_string()],
            )],
            fastn_core::http::StatusCode::OK,
        );
    }

    let (user, email) = user.expect("expected user to be Some");

    let key = generate_key(64);

    diesel::insert_into(fastn_core::schema::fastn_password_reset::table)
        .values((
            fastn_core::schema::fastn_password_reset::user_id.eq(&user.id),
            fastn_core::schema::fastn_password_reset::key.eq(&key),
            fastn_core::schema::fastn_password_reset::sent_at.eq(chrono::offset::Utc::now()),
        ))
        .execute(&mut conn)
        .await?;

    let reset_link = format!(
        "{}://{}/-/auth/reset-password/?code={key}?next={next}",
        req_config.request.connection_info.scheme(),
        req_config.request.connection_info.host(),
    );

    // To use auth. The package has to have auto import with alias `auth` setup
    let path = req_config
        .config
        .package
        .eval_auto_import("auth")
        .unwrap()
        .to_owned();

    let path = path
        .strip_prefix(format!("{}/", req_config.config.package.name).as_str())
        .unwrap();

    let content = req_config
        .config
        .ds
        .read_to_string(&fastn_ds::Path::new(format!("{}.ftd", path)))
        .await?;

    let auth_doc = fastn_core::Document {
        package_name: req_config.config.package.name.clone(),
        id: path.to_string(),
        content,
        parent_path: fastn_ds::Path::new("/"),
    };

    let main_ftd_doc = fastn_core::doc::interpret_helper(
        auth_doc.id_with_package().as_str(),
        auth_doc.content.as_str(),
        req_config,
        "/",
        false,
        0,
    )
    .await?;

    let html_email_templ = format!(
        "{}/{}#reset-password-request-mail-html",
        req_config.config.package.name, path
    );

    let html: String = main_ftd_doc.get(&html_email_templ).unwrap();
    let html = html.replace("{{link}}", &reset_link);

    tracing::info!("confirmation link: {}", &reset_link);

    fastn_core::mail::Mailer::send_raw(
        req_config
            .config
            .ds
            .env_bool("FASTN_ENABLE_EMAIL", true)
            .await,
        &req_config.config.ds,
        format!("{} <{}>", user.name, email.0)
            .parse::<lettre::message::Mailbox>()
            .unwrap(),
        "Reset your password",
        html,
    )
    .await
    .map_err(|e| fastn_core::Error::generic(format!("failed to send email: {e}")))?;

    let resp_body = serde_json::json!({
        "success": true,
        "redirect": redirect_url_from_next(&req_config.request, "/-/auth/forgot-password/".to_string()),
    });

    let mut resp = actix_web::HttpResponse::Ok();

    if req_config.config.test_command_running {
        resp.insert_header(("X-Fastn-Test", "true"))
            .insert_header(("X-Fastn-Test-Email-Confirmation-Link", reset_link));
    }

    Ok(resp.json(resp_body))
}

/// GET | POST /-/auth/reset-password/
/// setup set-password route
async fn reset_password(
    req_config: &mut fastn_core::RequestConfig,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let user_id = req_config
        .request
        .ud(&req_config.config.ds)
        .await
        .map(|u| format!("{}", u.id));

    let encrypted_user_id = if user_id.is_some() {
        // if user is authenticated, get user_id from there
        fastn_core::auth::utils::encrypt(&req_config.config.ds, &user_id.unwrap()).await
    } else {
        // use the ?code from query params, this is set in /-/auth/forgot-password/
        let key = req_config.request.query().get("code");

        if key.is_none() {
            return Ok(fastn_core::http::api_error("Bad Request")?);
        }

        let key = match key.unwrap() {
            serde_json::Value::String(c) => c.to_owned(),
            _ => {
                return Ok(fastn_core::http::api_error("Bad Request")?);
            }
        };

        let mut conn = db_pool
            .get()
            .await
            .map_err(|e| fastn_core::Error::DatabaseError {
                message: format!("Failed to get connection to db. {:?}", e),
            })?;

        let query = diesel::delete(
            fastn_core::schema::fastn_password_reset::table
                .filter(fastn_core::schema::fastn_password_reset::key.eq(&key)),
        )
        .returning(fastn_core::schema::fastn_password_reset::user_id);

        dbg!("{:?}", diesel::debug_query::<diesel::pg::Pg, _>(&query));

        let user_id: Option<i32> = query.get_result::<i32>(&mut conn).await.optional()?;

        if user_id.is_none() {
            return Ok(fastn_core::http::api_error("Bad Request")?);
        }

        let user_id = format!("{}", user_id.unwrap());

        fastn_core::auth::utils::encrypt(&req_config.config.ds, &user_id).await
    };

    let cookie = actix_web::cookie::Cookie::build("fastn_target_user_id", encrypted_user_id)
        .path("/-/auth/set-password")
        .same_site(actix_web::cookie::SameSite::Strict)
        .http_only(true)
        .secure(true)
        .finish();

    let resp = actix_web::HttpResponse::Ok()
        .cookie(cookie)
        .status(actix_web::http::StatusCode::TEMPORARY_REDIRECT)
        .insert_header((actix_web::http::header::LOCATION, format!("/-/auth/set-password?next={next}")))
        .finish();

    return Ok(resp);
}

// both forgot_password_request and reset_password_request will set some secure cookie that'll contain the fastn_target_user_id
/// GET | POST /-/auth/set-password/
/// read the cookie `fastn_target_user_id` and set the password
async fn set_password(
    req_config: &mut fastn_core::RequestConfig,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    if req_config.request.method() == "GET" {
        let main = fastn_core::Document {
            package_name: req_config.config.package.name.clone(),
            id: "/-/set-password".to_string(),
            content: set_password_form_ftd().to_string(),
            parent_path: fastn_ds::Path::new("/"),
        };

        let resp = fastn_core::package::package_doc::read_ftd(req_config, &main, "/", false, false)
            .await?;

        return Ok(resp.into());
    }

    if req_config.request.method() != "POST" {
        return Ok(fastn_core::not_found!("invalid route"));
    }

    #[derive(serde::Deserialize)]
    struct Payload {
        new_password: String,
        new_password2: String,
    }

    let payload = req_config.request.json::<Payload>();

    if let Err(e) = payload {
        return fastn_core::http::user_err(
            vec![
                ("payload".into(), vec![format!("invalid payload: {:?}", e)]),
                (
                    "new_password".into(),
                    vec!["new password is required".to_string()],
                ),
                (
                    "new_password2".into(),
                    vec!["confirm new password is required".to_string()],
                ),
            ],
            fastn_core::http::StatusCode::OK,
        );
    }

    let payload = payload.unwrap();

    if payload.new_password.is_empty() {
        return fastn_core::http::user_err(
            vec![(
                "new_password".into(),
                vec!["new password is required".to_string()],
            )],
            fastn_core::http::StatusCode::OK,
        );
    }

    if payload.new_password2.is_empty() {
        return fastn_core::http::user_err(
            vec![(
                "new_password2".into(),
                vec!["confirm new password is required".to_string()],
            )],
            fastn_core::http::StatusCode::OK,
        );
    }

    if payload.new_password != payload.new_password2 {
        return fastn_core::http::user_err(
            vec![(
                "new_password2".into(),
                vec!["new password and confirm new password do not match".to_string()],
            )],
            fastn_core::http::StatusCode::OK,
        );
    }

    let encrypted_user_id = match req_config 
        .request
        .cookie("fastn_target_user_id") {
            Some(v) => v,
            None => {
                return Ok(fastn_core::http::api_error("Bad Request")?);
            }
        };

    let user_id = fastn_core::auth::utils::decrypt(&req_config.config.ds, &encrypted_user_id).await
        .map(|v| v.parse::<i32>().expect("user_id must be an i32"))
        .map_err(
        |e| fastn_core::Error::generic(format!("decryption failed: {:?}", e)),
    )?;

    let mut conn = db_pool
        .get()
        .await
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to get connection to db. {:?}", e),
        })?;

    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);

    let argon2 = argon2::Argon2::default();

    let hashed_password =
        argon2::PasswordHasher::hash_password(&argon2, payload.new_password.as_bytes(), &salt)
            .map_err(|e| fastn_core::Error::generic(format!("error in hashing password: {e}")))?
            .to_string();

    diesel::update(fastn_core::schema::fastn_user::table)
        .set(fastn_core::schema::fastn_user::password.eq(&hashed_password))
        .filter(fastn_core::schema::fastn_user::id.eq(&user_id))
        .execute(&mut conn)
        .await?;
}
