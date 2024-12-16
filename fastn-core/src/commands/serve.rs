use fastn_core::http::Response;

#[tracing::instrument(skip_all)]
fn handle_redirect(config: &fastn_core::Config, path: &camino::Utf8Path) -> Option<Response> {
    config
        .package
        .redirects
        .as_ref()
        .and_then(|v| fastn_core::package::redirects::find_redirect(v, path.as_str()))
        .map(|r| Response::PermanentRedirect {
            location: r.to_string(),
        })
}

/// path: /-/<package-name>/<file-name>/
/// path: /<file-name>/
#[tracing::instrument(skip_all)]
async fn serve_file(
    config: &mut fastn_core::RequestConfig,
    path: &camino::Utf8Path,
    only_js: bool,
    preview_session_id: &Option<String>,
) -> fastn_core::Result<Response> {
    config
        .config
        .package
        .auto_import_language(config.request.cookie("fastn-lang"), None)?;

    let f = config
        .get_file_and_package_by_id(path.as_str(), preview_session_id)
        .await?;

    if let fastn_core::File::Code(doc) = f {
        let path = doc.get_full_path().to_string();
        let mime = Some(mime_guess::from_path(path).first_or_text_plain());
        return Ok(Response::Raw {
            content: doc.content.into_bytes(),
            mime,
            headers: None,
            cookies: None,
        });
    }

    let main_document = match f {
        fastn_core::File::Ftd(main_document) => main_document,
        _ => {
            tracing::error!(msg = "unknown handler", path = path.as_str());
            tracing::info!("file: {f:?}");
            return Err(fastn_core::Error::UnknownHandler);
        }
    };

    fastn_core::package::package_doc::read_ftd_(
        config,
        &main_document,
        "/",
        false,
        false,
        only_js,
        preview_session_id,
    )
    .await
    .map(|res| Response::FTDResult { res, cookies: None })
}

fn guess_mime_type(path: &str) -> mime_guess::Mime {
    mime_guess::from_path(path).first_or_octet_stream()
}

pub fn clear_sid(req: &fastn_core::http::Request) -> actix_web::HttpResponse {
    let mut cookie = actix_web::cookie::Cookie::build(ft_sys_shared::SESSION_KEY, "")
        .domain(match req.connection_info.host().split_once(':') {
            Some((domain, _port)) => domain.to_string(),
            None => req.connection_info.host().to_string(),
        })
        .path("/")
        .max_age(actix_web::cookie::time::Duration::seconds(34560000))
        .secure(true)
        .same_site(actix_web::cookie::SameSite::Strict)
        .finish();
    cookie.make_removal();

    dbg!(
        actix_web::HttpResponse::build(actix_web::http::StatusCode::TEMPORARY_REDIRECT)
            .insert_header(("LOCATION", "/"))
            .cookie(cookie)
            .finish()
    )
}

pub fn clear_sid2(req: &fastn_core::http::Request) -> Response {
    // safari is ignoring cookie if we return a redirect, so we are returning a meta-refresh
    // further we are not using .secure(true) here because then cookie is not working on
    // localhost

    let cookie = actix_web::cookie::Cookie::build(ft_sys_shared::SESSION_KEY, "")
        .domain(match req.connection_info.host().split_once(':') {
            Some((domain, _port)) => domain.to_string(),
            None => req.connection_info.host().to_string(),
        })
        .path("/")
        .max_age(actix_web::cookie::time::Duration::seconds(0))
        .same_site(actix_web::cookie::SameSite::Strict)
        .finish();

    Response::Raw {
        content: r#"<meta http-equiv="refresh" content="0; url=/" />"#.into(),
        headers: Some(vec![("Content-Type".to_string(), "text/html".to_string())]),
        cookies: Some(vec![cookie.to_string()]),
        mime: None,
    }
}

/// Wraps [serve] to send [actix_web::HttpResponse]
pub async fn actix_serve(
    config: &fastn_core::Config,
    req: fastn_core::http::Request,
    only_js: bool,
    preview_session_id: &Option<String>,
) -> fastn_core::Result<(actix_web::HttpResponse, bool)> {
    let (res, cacheable) = serve(config, req, only_js, preview_session_id).await?;

    let resp = match res {
        Response::FTDResult { res, cookies } => {
            match res {
                fastn_core::package::package_doc::FTDResult::Html(body) => {
                    fastn_core::http::ok_with_content_type(body, mime_guess::mime::TEXT_HTML_UTF_8)
                }
                fastn_core::package::package_doc::FTDResult::Redirect { url, code } => {
                    if Some(mime_guess::mime::APPLICATION_JSON) == req.content_type() {
                        fastn_core::http::ok_with_content_type(
                            // intentionally using `.unwrap()` as this should never fail
                            serde_json::to_vec(&serde_json::json!({ "redirect": url })).unwrap(),
                            mime_guess::mime::APPLICATION_JSON,
                        )
                    } else {
                        fastn_core::http::redirect_with_code(url, code)
                    }
                }
            }
        }
        Response::Raw {
            content,
            mime,
            cookies,
            headers,
        } => {
            let mut resp = &mut actix_web::HttpResponse::Ok();

            if let Some(mime) = mime {
                resp = resp.content_type(mime);
            }

            let resp = resp.body(content);

            if let Some(cookies) = cookies {
                for cookie in cookies {
                    resp.headers_mut().append(
                        actix_web::http::header::SET_COOKIE,
                        actix_web::http::header::HeaderValue::from_str(cookie.as_str()).unwrap(),
                    );
                }
            }

            if let Some(headers) = headers {
                for (k, v) in headers {
                    resp.headers_mut().insert(
                        k.parse().unwrap(),
                        actix_web::http::header::HeaderValue::from_str(v.as_str()).unwrap(),
                    );
                }
            }

            resp
        }
        Response::PermanentRedirect { location } => {
            fastn_core::http::permanent_redirect(location)
        }
        Response::Wasm { request } => fastn_ds::wasm::to_response(request),
        Response::Reqwest { response } => {
            fastn_core::http::ResponseBuilder::from_reqwest(response).await
        }
        Response::DefaultRoute { content, mime } => actix_web::HttpResponse::Ok()
            .content_type(mime)
            .append_header(("Cache-Control", "public, max-age=31536000"))
            .body(content),
        Response::NotFound { message } => fastn_core::http::not_found_without_warning(message),
    };

    Ok((resp, cacheable))
}

#[tracing::instrument(skip_all)]
pub async fn serve(
    config: &fastn_core::Config,
    req: fastn_core::http::Request,
    only_js: bool,
    preview_session_id: &Option<String>,
) -> fastn_core::Result<(Response, bool)> {
    let mut req_config = fastn_core::RequestConfig::new(config, &req, "", "/");

    if req.path() == "/-/auth/logout/" {
        return Ok((clear_sid2(&req), false));
    }

    if let Some(endpoint_response) = handle_endpoints(config, &req, preview_session_id).await {
        return endpoint_response.map(|r| (r, false));
    }

    if let Some(app_response) = handle_apps(config, &req).await {
        return app_response.map(|r| (r, false));
    }

    if let Some(default_response) = handle_default_route(&req, config.package.name.as_str()) {
        return Ok((default_response, true));
    }

    let path: camino::Utf8PathBuf = req.path().replacen('/', "", 1).parse()?;

    if let Some(r) = handle_redirect(config, &path) {
        return Ok((r, false));
    }

    if fastn_core::utils::is_static_path(req.path()) {
        return handle_static_route(
            req.path(),
            config.package.name.as_str(),
            &config.ds,
            preview_session_id,
        )
        .await
        .map(|r| (r, true));
    }

    serve_helper(&mut req_config, only_js, path, preview_session_id)
        .await
        .map(|r| (r, req_config.response_is_cacheable))
}

#[tracing::instrument(skip_all)]
pub async fn serve_helper(
    req_config: &mut fastn_core::RequestConfig,
    only_js: bool,
    path: camino::Utf8PathBuf,
    preview_session_id: &Option<String>,
) -> fastn_core::Result<Response> {
    let mut resp = if req_config.request.path() == "/" {
        serve_file(req_config, &path.join("/"), only_js, preview_session_id).await
    } else {
        // url is present in config or not
        // If not present than proxy pass it

        let query_string = req_config.request.query_string().to_string();

        // if start with -/ and mount-point exists so send redirect to mount-point
        // We have to do -/<package-name>/remaining-url/ ==> (<package-name>, remaining-url) ==> (/config.package-name.mount-point/remaining-url/)
        // Get all the dependencies with mount-point if path_start with any package-name so send redirect to mount-point
        // fastn_core::file::is_static: checking for static file, if file is static no need to redirect it.
        // if any app name starts with package-name to redirect it to /mount-point/remaining-url/
        for (mp, dep) in req_config
            .config
            .package
            .apps
            .iter()
            .map(|x| (&x.mount_point, &x.package))
        {
            if let Some(remaining_path) =
                fastn_core::config::utils::trim_package_name(path.as_str(), dep.name.as_str())
            {
                let path = if remaining_path.trim_matches('/').is_empty() {
                    format!("/{}/", mp.trim().trim_matches('/'))
                } else if query_string.is_empty() {
                    format!(
                        "/{}/{}/",
                        mp.trim().trim_matches('/'),
                        remaining_path.trim_matches('/')
                    )
                } else {
                    format!(
                        "/{}/{}/?{}",
                        mp.trim().trim_matches('/'),
                        remaining_path.trim_matches('/'),
                        query_string.as_str()
                    )
                };

                return Ok(Response::PermanentRedirect { location: path });
            }
        }

        serve_file(req_config, path.as_path(), only_js, preview_session_id).await
    };

    if let Some(request) = req_config.processor_set_response.take() {
        return Ok(Response::Wasm { request });
    }

    if !req_config.processor_set_cookies.is_empty() {
        resp.as_mut()
            .map(|ref mut x| x.attach_cookies(req_config.processor_set_cookies.clone()));
    }

    resp
}

pub fn handle_default_route(
    req: &fastn_core::http::Request,
    package_name: &str,
) -> Option<Response> {
    if req
        .path()
        .ends_with(fastn_core::utils::hashed_default_css_name())
    {
        Some(Response::DefaultRoute {
            content: ftd::css().to_string(),
            mime: mime_guess::mime::TEXT_CSS,
        })
    } else if req
        .path()
        .ends_with(fastn_core::utils::hashed_default_js_name())
    {
        Some(Response::DefaultRoute {
            content: format!("{}\n\n{}", ftd::build_js(), fastn_core::fastn_2022_js()),
            mime: mime_guess::mime::TEXT_JAVASCRIPT,
        })
    } else if req
        .path()
        .ends_with(fastn_core::utils::hashed_default_ftd_js(package_name))
    {
        Some(Response::DefaultRoute {
            content: ftd::js::all_js_without_test(package_name),
            mime: mime_guess::mime::TEXT_JAVASCRIPT,
        })
    } else if req
        .path()
        .ends_with(fastn_core::utils::hashed_markdown_js())
    {
        Some(Response::DefaultRoute {
            content: ftd::markdown_js().to_string(),
            mime: mime_guess::mime::TEXT_JAVASCRIPT,
        })
    } else if req.path().ends_with(fastn_core::utils::hashed_prism_js()) {
        Some(Response::DefaultRoute {
            content: ftd::prism_js().to_string(),
            mime: mime_guess::mime::TEXT_JAVASCRIPT,
        })
    } else if req.path().ends_with(fastn_core::utils::hashed_prism_css()) {
        Some(Response::DefaultRoute {
            content: ftd::prism_css().to_string(),
            mime: mime_guess::mime::TEXT_CSS,
        })
    } else {
        None
    }
}

async fn handle_static_route(
    path: &str,
    package_name: &str,
    ds: &fastn_ds::DocumentStore,
    session_id: &Option<String>,
) -> fastn_core::Result<Response> {
    return match handle_static_route_(path, package_name, ds, session_id).await {
        Ok(r) => Ok(r),
        Err(fastn_ds::ReadError::NotFound(_)) => {
            handle_not_found_image(path, package_name, ds, session_id).await
        }
        Err(e) => Err(e.into()),
    };

    async fn handle_static_route_(
        path: &str,
        package_name: &str,
        ds: &fastn_ds::DocumentStore,
        session_id: &Option<String>,
    ) -> Result<Response, fastn_ds::ReadError> {
        if path == "/favicon.ico" {
            return favicon(ds, session_id).await;
        }

        // the path can start with slash or -/. If later, it is a static file from our dependencies, so
        // we have to look for them inside .packages.
        let path = match path.strip_prefix("/-/") {
            Some(path) if path.starts_with(package_name) => {
                path.strip_prefix(package_name).unwrap_or(path).to_string()
            }
            Some(path) => format!(".packages/{path}"),
            None => path.to_string(),
        };

        static_file(
            ds,
            path.strip_prefix('/').unwrap_or(path.as_str()),
            session_id,
        )
        .await
        .map_err(Into::into)
    }

    async fn handle_not_found_image(
        path: &str,
        package_name: &str,
        ds: &fastn_ds::DocumentStore,
        session_id: &Option<String>,
    ) -> fastn_core::Result<Response> {
        // todo: handle dark images using manifest
        if let Some(new_file_path) = generate_dark_image_path(path) {
            return handle_static_route_(new_file_path.as_str(), package_name, ds, session_id)
                .await
                .or_else(|e| {
                    if let fastn_ds::ReadError::NotFound(e) = e {
                        Ok(Response::NotFound { message: e })
                    } else {
                        Err(e.into())
                    }
                });
        }

        Ok(Response::NotFound {
            message: "".to_string(),
        })
    }

    fn generate_dark_image_path(path: &str) -> Option<String> {
        match path.rsplit_once('.') {
            Some((remaining, ext))
                if mime_guess::MimeGuess::from_ext(ext)
                    .first_or_octet_stream()
                    .to_string()
                    .starts_with("image/") =>
            {
                Some(if remaining.ends_with("-dark") {
                    format!("{}.{}", remaining.trim_end_matches("-dark"), ext)
                } else {
                    format!("{}-dark.{}", remaining, ext)
                })
            }
            _ => None,
        }
    }

    async fn favicon(
        ds: &fastn_ds::DocumentStore,
        session_id: &Option<String>,
    ) -> Result<Response, fastn_ds::ReadError> {
        match static_file(ds, "favicon.ico", session_id).await {
            Ok(r) => Ok(r),
            Err(fastn_ds::ReadError::NotFound(_)) => {
                Ok(static_file(ds, "static/favicon.ico", session_id).await?)
            }
            Err(e) => Err(e),
        }
    }

    #[tracing::instrument(skip(ds))]
    async fn static_file(
        ds: &fastn_ds::DocumentStore,
        path: &str,
        session_id: &Option<String>,
    ) -> Result<Response, fastn_ds::ReadError> {
        ds.read_content(&fastn_ds::Path::new(path), session_id)
            .await
            .map(|r| Response::Raw {
                content: r,
                mime: Some(guess_mime_type(path.to_string().as_str())),
                cookies: None,
                headers: None,
            })
    }
}

async fn handle_endpoints(
    config: &fastn_core::Config,
    req: &fastn_core::http::Request,
    session_id: &Option<String>,
) -> Option<fastn_core::Result<Response>> {
    let matched_endpoint = config
        .package
        .endpoints
        .iter()
        .find(|ep| req.path().starts_with(ep.mountpoint.trim_end_matches('/')));

    let endpoint = match matched_endpoint {
        Some(e) => e,
        None => return None,
    };

    let url = format!(
        "{}/{}",
        endpoint.endpoint.trim_end_matches('/'),
        req.full_path()
            .strip_prefix(endpoint.mountpoint.trim_end_matches('/'))
            .map(|v| v.trim_start_matches('/'))
            .expect("req.full_path() must start with endpoint.mountpoint")
    );

    if url.starts_with("wasm+proxy://") {
        return match config
            .ds
            .handle_wasm(url, req, endpoint.mountpoint.to_string(), session_id)
            .await
        {
            Ok(r) => Some(Ok(Response::Wasm { request: r })),
            Err(e) => return Some(Err(e.into())),
        };
    }

    let response = match config
        .ds
        .http(
            url::Url::parse(url.as_str()).unwrap(),
            req,
            &std::collections::HashMap::new(),
        )
        .await
        .map_err(fastn_core::Error::DSHttpError)
    {
        Ok(response) => Response::Reqwest { response },
        Err(e) => return Some(Err(e)),
    };

    Some(Ok(response))
}

async fn handle_apps(
    config: &fastn_core::Config,
    req: &fastn_core::http::Request,
) -> Option<fastn_core::Result<Response>> {
    let matched_app = config.package.apps.iter().find(|a| {
        req.path().starts_with(
            a.end_point
                .clone()
                .unwrap_or_default()
                .trim_end_matches('/'),
        )
    });

    let _app = match matched_app {
        Some(e) => e,
        None => return None,
    };

    // app.package.endpoints
    // app.package.apps

    // see if app.pack

    None
}

#[tracing::instrument(skip_all)]
async fn actual_route(
    config: &fastn_core::Config,
    req: actix_web::HttpRequest,
    body: actix_web::web::Bytes,
    preview_session_id: &Option<String>,
) -> fastn_core::Result<actix_web::HttpResponse> {
    tracing::info!(method = req.method().as_str(), uri = req.path());
    let req = fastn_core::http::Request::from_actix(req, body);

    actix_serve(config, req, false, preview_session_id)
        .await
        .map(|(r, _)| r)
}

#[tracing::instrument(skip_all)]
async fn route(
    req: actix_web::HttpRequest,
    body: actix_web::web::Bytes,
    config: actix_web::web::Data<std::sync::Arc<fastn_core::Config>>,
) -> fastn_core::Result<actix_web::HttpResponse> {
    actual_route(&config, req, body, &None).await
}

#[allow(clippy::too_many_arguments)]
pub async fn listen(
    config: std::sync::Arc<fastn_core::Config>,
    bind_address: &str,
    port: Option<u16>,
) -> fastn_core::Result<()> {
    use colored::Colorize;
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let tcp_listener = match fastn_core::http::get_available_port(port, bind_address) {
        Some(listener) => listener,
        None => {
            eprintln!(
                "{}",
                port.map(|x| format!(
                    r#"Provided port {} is not available.

You can try without providing port, it will automatically pick unused port."#,
                    x.to_string().red()
                ))
                .unwrap_or_else(|| {
                    "Tried picking port between port 8000 to 9000, none are available :-("
                        .to_string()
                })
            );
            std::process::exit(2);
        }
    };

    let app = move || {
        actix_web::App::new()
            .app_data(actix_web::web::Data::new(std::sync::Arc::clone(&config)))
            .app_data(actix_web::web::PayloadConfig::new(1024 * 1024 * 10))
            .wrap(actix_web::middleware::Compress::default())
            .wrap(fastn_core::catch_panic::CatchPanic::default())
            .wrap(
                actix_web::middleware::Logger::new(
                    r#""%r" %Ts %s %b %a "%{Referer}i" "%{User-Agent}i""#,
                )
                .log_target(""),
            )
            .route("/{path:.*}", actix_web::web::route().to(route))
    };

    println!("### Server Started ###");
    println!(
        "Go to: http://{}:{}",
        bind_address,
        tcp_listener.local_addr()?.port()
    );
    actix_web::HttpServer::new(app)
        .listen(tcp_listener)?
        .run()
        .await?;
    Ok(())
}
