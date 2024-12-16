pub const SESSION_COOKIE_NAME: &str = "fastn-sid";
pub const X_FASTN_REQUEST_PATH: &str = "x-fastn-request-path";
pub const X_FASTN_ROOT: &str = "x-fastn-root";

pub enum Response {
    /// FTDResult is the result of processing an FTD file using [fastn_core::serve_file]
    FTDResult {
        res: fastn_core::package::package_doc::FTDResult,
        /// any cookies set via the `http` processor
        cookies: Option<Vec<String>>
    },
    Raw {
        content: Vec<u8>,
        mime: Option<mime_guess::Mime>,
        cookies: Option<Vec<String>>,
        headers: Option<Vec<(String, String)>>,
    },
    PermanentRedirect {
        location: String,
    },
    Wasm {
        request: ft_sys_shared::Request,
    },
    Reqwest {
        response: http::Response<bytes::Bytes>,
    },
    /// DefaultRoute is the result of [fastn_core::handle_default_route]
    /// Http Cache-Control header should be applied to this.
    DefaultRoute {
        content: String,
        mime: mime_guess::Mime,
    },
    NotFound {
        message: String,
    }
}

impl Response {
    pub fn attach_cookies(&mut self, cookies: Vec<String>) {
        let c = cookies;
        match self {
            Response::FTDResult { ref mut cookies, .. } => {
                *cookies = Some(c);
            }
            Response::Raw { ref mut cookies, .. } => {
                *cookies = Some(c);
            }
            _ => {
                panic!("cookies can only be attached to FTDResult");
            }
        }
    }
}


#[macro_export]
macro_rules! server_error {
    ($($t:tt)*) => {{
        fastn_core::http::server_error_(format!($($t)*))
    }};
}

#[macro_export]
macro_rules! not_found {
    ($($t:tt)*) => {{
        fastn_core::http::not_found_(format!($($t)*) + "\n")
    }};
}

#[macro_export]
macro_rules! unauthorised {
    ($($t:tt)*) => {{
        fastn_core::http::unauthorised_(format!($($t)*))
    }};
}

pub fn api_ok(data: impl serde::Serialize) -> serde_json::Result<actix_web::HttpResponse> {
    #[derive(serde::Serialize)]
    struct SuccessResponse<T: serde::Serialize> {
        data: T,
        success: bool,
    }

    let data = serde_json::to_vec(&SuccessResponse {
        data,
        success: true,
    })?;

    Ok(ok_with_content_type(
        data,
        mime_guess::mime::APPLICATION_JSON,
    ))
}

pub fn unauthorised_(msg: String) -> actix_web::HttpResponse {
    fastn_core::warning!("unauthorised: {}", msg);
    actix_web::HttpResponse::Unauthorized().body(msg)
}

pub fn server_error_(msg: String) -> actix_web::HttpResponse {
    fastn_core::warning!("server error: {}", msg);
    server_error_without_warning(msg)
}

pub fn server_error_without_warning(msg: String) -> actix_web::HttpResponse {
    actix_web::HttpResponse::InternalServerError().body(msg)
}

pub fn not_found_without_warning(msg: String) -> actix_web::HttpResponse {
    actix_web::HttpResponse::NotFound().body(msg)
}

pub fn not_found_(msg: String) -> actix_web::HttpResponse {
    fastn_core::warning!("page not found: {}", msg);
    not_found_without_warning(msg)
}

impl actix_web::ResponseError for fastn_core::Error {}

pub type StatusCode = actix_web::http::StatusCode;

pub fn permanent_redirect(url: String) -> actix_web::HttpResponse {
    redirect_with_code(url, 308)
}

pub fn redirect_with_code(url: String, code: u16) -> actix_web::HttpResponse {
    match code {
        301 => actix_web::HttpResponse::MovedPermanently(),
        302 => actix_web::HttpResponse::Found(),
        303 => actix_web::HttpResponse::SeeOther(),
        307 => actix_web::HttpResponse::TemporaryRedirect(),
        308 => actix_web::HttpResponse::PermanentRedirect(),
        _ => {
            fastn_core::warning!("invalid redirect code: {code}");
            actix_web::HttpResponse::PermanentRedirect()
        }
    }
    .insert_header(("LOCATION", url))
    .finish()
}

pub fn ok_with_content_type(
    data: Vec<u8>,
    content_type: mime_guess::Mime,
) -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok()
        .content_type(content_type)
        .body(data)
}

pub(crate) struct ResponseBuilder {}

impl ResponseBuilder {
    #[allow(dead_code)]
    pub async fn from_reqwest(response: http::Response<bytes::Bytes>) -> actix_web::HttpResponse {
        let status = response.status().as_u16();

        // Remove `Connection` as per
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
        let mut response_builder =
            actix_web::HttpResponse::build(actix_web::http::StatusCode::from_u16(status).unwrap());
        for (k, v) in response
            .headers()
            .iter()
            .filter(|(h, _)| *h != "connection")
        {
            response_builder.insert_header((k.as_str(), v.as_bytes()));
        }

        let content = response.body();
        response_builder.body(content.to_vec())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Request {
    host: String,
    method: String,
    pub uri: String,
    pub path: String,
    query_string: String,
    cookies: std::collections::HashMap<String, String>,
    headers: reqwest::header::HeaderMap,
    query: std::collections::HashMap<String, serde_json::Value>,
    body: actix_web::web::Bytes,
    ip: Option<String>,
    pub connection_info: actix_web::dev::ConnectionInfo,
    // path_params: Vec<(String, )>
}

#[async_trait::async_trait]
impl fastn_ds::RequestType for Request {
    fn headers(&self) -> &reqwest::header::HeaderMap {
        &self.headers
    }

    fn method(&self) -> &str {
        self.method.as_str()
    }

    fn query_string(&self) -> &str {
        self.query_string.as_str()
    }

    fn get_ip(&self) -> Option<String> {
        self.ip.clone()
    }

    fn cookies_string(&self) -> Option<String> {
        if self.cookies.is_empty() {
            return None;
        }
        Some(
            self.cookies()
                .iter()
                // TODO: check if extra escaping is needed
                .map(|(k, v)| format!("{}={}", k, v).replace(';', "%3B"))
                .collect::<Vec<_>>()
                .join(";"),
        )
    }

    fn body(&self) -> &[u8] {
        &self.body
    }
}

impl Request {
    pub fn from_actix(req: actix_web::HttpRequest, body: actix_web::web::Bytes) -> Self {
        let headers = {
            let mut headers = reqwest::header::HeaderMap::new();
            for (key, value) in req.headers() {
                if let (Ok(v), Ok(k)) = (
                    value.to_str().unwrap_or("").parse::<http::HeaderValue>(),
                    http::HeaderName::from_bytes(key.as_str().as_bytes()),
                ) {
                    headers.insert(k, v.clone());
                } else {
                    tracing::warn!("failed to parse header: {key:?} {value:?}");
                }
            }
            headers
        };

        return Request {
            cookies: get_cookies(&headers),
            body,
            host: req.connection_info().host().to_string(),
            method: req.method().to_string(),
            uri: req.uri().to_string(),
            path: req.path().to_string(),
            query_string: req.query_string().to_string(),
            connection_info: req.connection_info().clone(),
            headers,
            query: {
                actix_web::web::Query::<std::collections::HashMap<String, serde_json::Value>>::from_query(
                    req.query_string(),
                ).unwrap().0
            },
            ip: req.peer_addr().map(|x| x.ip().to_string()),
        };

        fn get_cookies(
            headers: &reqwest::header::HeaderMap,
        ) -> std::collections::HashMap<String, String> {
            let mut cookies = std::collections::HashMap::new();
            if let Some(cookie) = headers.get("cookie") {
                if let Ok(cookie) = cookie.to_str() {
                    for cookie in cookie.split(';') {
                        let cookie = cookie.trim();
                        if let Some(index) = cookie.find('=') {
                            let (key, value) = cookie.split_at(index);
                            let key = key.trim();
                            let value = value.trim_start_matches('=').trim();
                            cookies.insert(key.to_string(), value.to_string());
                        }
                    }
                }
            }
            cookies
        }
    }

    pub fn full_path(&self) -> String {
        if self.query_string.is_empty() {
            self.path.clone()
        } else {
            format!("{}?{}", self.path, self.query_string)
        }
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }

    pub fn method(&self) -> &str {
        self.method.as_str()
    }

    pub fn uri(&self) -> &str {
        self.uri.as_str()
    }

    pub fn body_as_json(
        &self,
    ) -> fastn_core::Result<Option<std::collections::HashMap<String, serde_json::Value>>> {
        if self.body.is_empty() {
            return Ok(None);
        }
        if self.content_type() != Some(mime_guess::mime::APPLICATION_JSON) {
            return Err(fastn_core::Error::UsageError {
                message: fastn_core::warning!(
                    "expected content type {}, got {:?}",
                    mime_guess::mime::APPLICATION_JSON,
                    self.content_type()
                ),
            });
        }
        Ok(Some(serde_json::from_slice(&self.body)?))
    }

    pub fn x_fastn_request_path(&self) -> Option<String> {
        self.headers
            .get(X_FASTN_REQUEST_PATH)
            .and_then(|v| v.to_str().map(|v| v.to_string()).ok())
    }

    pub fn x_fastn_root(&self) -> Option<String> {
        self.headers
            .get(X_FASTN_ROOT)
            .and_then(|v| v.to_str().map(|v| v.to_string()).ok())
    }

    pub fn content_type(&self) -> Option<mime_guess::Mime> {
        self.headers
            .get(actix_web::http::header::CONTENT_TYPE.as_str())
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
    }

    pub fn user_agent(&self) -> Option<String> {
        self.headers
            .get(actix_web::http::header::USER_AGENT.as_str())
            .and_then(|v| v.to_str().map(|v| v.to_string()).ok())
    }

    pub fn headers(&self) -> &reqwest::header::HeaderMap {
        &self.headers
    }

    pub fn json<T: serde::de::DeserializeOwned>(&self) -> serde_json::Result<T> {
        serde_json::from_slice(&self.body)
    }

    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    pub fn query_string(&self) -> &str {
        self.query_string.as_str()
    }

    pub fn cookies(&self) -> &std::collections::HashMap<String, String> {
        &self.cookies
    }

    pub fn cookie(&self, name: &str) -> Option<String> {
        self.cookies().get(name).map(|v| v.to_string())
    }

    pub fn set_body(&mut self, body: actix_web::web::Bytes) {
        self.body = body;
    }

    pub fn set_cookies(&mut self, cookies: &std::collections::HashMap<String, String>) {
        self.cookies.clone_from(cookies);
    }

    pub fn set_ip(&mut self, ip: Option<String>) {
        self.ip = ip;
    }

    pub fn set_headers(&mut self, headers: &std::collections::HashMap<String, String>) {
        for (key, value) in headers.iter() {
            self.headers.insert(
                reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                reqwest::header::HeaderValue::from_str(value.as_str()).unwrap(),
            );
        }
    }

    pub fn set_x_fastn_request_path(&mut self, value: &str) {
        self.headers.insert(
            reqwest::header::HeaderName::from_bytes(X_FASTN_REQUEST_PATH.as_bytes()).unwrap(),
            reqwest::header::HeaderValue::from_str(value).unwrap(),
        );
    }

    pub fn set_x_fastn_root(&mut self, value: &str) {
        self.headers.insert(
            reqwest::header::HeaderName::from_bytes(X_FASTN_ROOT.as_bytes()).unwrap(),
            reqwest::header::HeaderValue::from_str(value).unwrap(),
        );
    }

    pub fn set_method(&mut self, method: &str) {
        self.method = method.to_uppercase();
    }

    pub fn set_query_string(&mut self, query_string: &str) {
        self.query_string = query_string.to_string();
        self.query = actix_web::web::Query::<std::collections::HashMap<String, serde_json::Value>>::from_query(
            self.query_string.as_str(),
        ).unwrap().0;
    }

    pub fn query(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.query
    }

    pub fn host(&self) -> String {
        self.host.to_string()
    }

    pub fn is_bot(&self) -> bool {
        match self.user_agent() {
            Some(user_agent) => is_bot(&user_agent),
            None => true,
        }
    }
}

pub(crate) fn url_regex() -> regex::Regex {
    regex::Regex::new(
        r"((([A-Za-z]{3,9}:(?://)?)(?:[-;:&=\+\$,\w]+@)?[A-Za-z0-9.-]+|(?:www.|[-;:&=\+\$,\w]+@)[A-Za-z0-9.-]+)((?:/[\+~%/.\w_]*)?\??(?:[-\+=&;%@.\w_]*)\#?(?:[\w]*))?)"
    ).unwrap()
}

#[tracing::instrument(skip(req_config, headers, body))]
pub async fn http_post_with_cookie(
    req_config: &fastn_core::RequestConfig,
    url: &str,
    headers: &std::collections::HashMap<String, String>,
    body: &str,
) -> fastn_core::Result<(fastn_core::Result<bytes::Bytes>, Vec<String>)> {
    pub use fastn_ds::RequestType;

    let cookies = req_config.request.cookies().clone();
    let mut http_request = fastn_core::http::Request::default();
    http_request.set_method("post");
    http_request.set_cookies(&cookies);
    http_request.set_headers(headers);
    http_request.set_ip(req_config.request.ip.clone());
    http_request.set_body(actix_web::web::Bytes::copy_from_slice(body.as_bytes()));
    http_request.set_x_fastn_request_path(req_config.request.path.as_str());
    http_request.set_x_fastn_root(req_config.config.ds.root_str().as_str());

    let http_url = url::Url::parse(url).map_err(|e| fastn_core::Error::DSHttpError(e.into()))?;
    let res = req_config
        .config
        .ds
        .http(http_url, &http_request, &Default::default())
        .await
        .map_err(fastn_core::Error::DSHttpError)?;

    let mut resp_cookies = vec![];
    res.headers().iter().for_each(|(k, v)| {
        if k.as_str().eq("set-cookie") {
            if let Ok(v) = v.to_str() {
                resp_cookies.push(v.to_string());
            }
        }
    });

    if !res.status().eq(&http::StatusCode::OK) {
        let message = format!(
            "url: {}, response_status: {}, response: {:?}",
            url,
            res.status(),
            res.body()
        );

        tracing::error!(url = url, msg = message);
        return Ok((
            Err(fastn_core::Error::APIResponseError(message)),
            resp_cookies,
        ));
    }
    Ok((Ok(res.body().clone()), resp_cookies))
}

pub async fn http_get(ds: &fastn_ds::DocumentStore, url: &str) -> fastn_core::Result<bytes::Bytes> {
    tracing::debug!("http_get {}", &url);

    http_get_with_cookie(ds, &Default::default(), url, &Default::default(), true)
        .await?
        .0
}

static NOT_FOUND_CACHE: once_cell::sync::Lazy<antidote::RwLock<std::collections::HashSet<String>>> =
    once_cell::sync::Lazy::new(|| antidote::RwLock::new(Default::default()));

#[tracing::instrument(skip(ds, req, headers))]
pub async fn http_get_with_cookie(
    ds: &fastn_ds::DocumentStore,
    req: &fastn_core::http::Request,
    url: &str,
    headers: &std::collections::HashMap<String, String>,
    use_cache: bool,
) -> fastn_core::Result<(fastn_core::Result<bytes::Bytes>, Vec<String>)> {
    pub use fastn_ds::RequestType;
    if use_cache && NOT_FOUND_CACHE.read().contains(url) {
        return Ok((
            Err(fastn_core::Error::APIResponseError(
                "page not found, cached".to_string(),
            )),
            vec![],
        ));
    }

    let mut http_request = fastn_core::http::Request::default();
    http_request.set_method("get");
    http_request.set_cookies(req.cookies());
    http_request.set_headers(headers);
    http_request.set_x_fastn_request_path(req.path.as_str());
    http_request.set_x_fastn_root(ds.root_str().as_str());
    http_request.set_ip(req.ip.clone());
    let http_url = url::Url::parse(url).map_err(|e| fastn_core::Error::DSHttpError(e.into()))?;
    let res = ds
        .http(http_url, &http_request, &Default::default())
        .await
        .map_err(fastn_core::Error::DSHttpError)?;

    let mut resp_cookies = vec![];
    res.headers().iter().for_each(|(k, v)| {
        if k.as_str().eq("set-cookie") {
            if let Ok(v) = v.to_str() {
                resp_cookies.push(v.to_string());
            }
        }
    });

    if !res.status().eq(&http::StatusCode::OK) {
        let message = format!(
            "url: {}, response_status: {}, response: {:?}",
            url,
            res.status(),
            res
        );

        if use_cache {
            NOT_FOUND_CACHE.write().insert(url.to_string());
        }

        tracing::error!(url = url, msg = message);
        return Ok((
            Err(fastn_core::Error::APIResponseError(message)),
            resp_cookies,
        ));
    }

    Ok((Ok(res.body().clone()), resp_cookies))
}

pub(crate) fn get_available_port(
    port: Option<u16>,
    bind_address: &str,
) -> Option<std::net::TcpListener> {
    let available_listener =
        |port: u16, bind_address: &str| std::net::TcpListener::bind((bind_address, port));

    if let Some(port) = port {
        return match available_listener(port, bind_address) {
            Ok(l) => Some(l),
            Err(_) => None,
        };
    }

    for x in 8000..9000 {
        match available_listener(x, bind_address) {
            Ok(l) => return Some(l),
            Err(_) => continue,
        }
    }
    None
}

/// construct `ftd.http` consumable error responses
/// https://github.com/fastn-stack/fastn/blob/7f0b79a/fastn-js/js/ftd.js#L218C45-L229
/// ```json
/// {
///     "data": null,
///     "errors": {
///         key: String,
///         key2: String,
///         ...
///     }
/// }
/// ```
pub fn user_err(
    errors: Vec<(String, Vec<String>)>,
    status_code: fastn_core::http::StatusCode,
) -> fastn_core::Result<actix_web::HttpResponse> {
    let mut json_error = serde_json::Map::new();

    for (k, v) in errors {
        let messages =
            serde_json::Value::Array(v.into_iter().map(serde_json::Value::String).collect());
        json_error.insert(k.to_string(), messages);
    }

    let resp = serde_json::json!({
        "success": false,
        "errors": json_error,
    });

    Ok(actix_web::HttpResponse::Ok()
        .status(status_code)
        .content_type(actix_web::http::header::ContentType::json())
        .body(serde_json::to_string(&resp)?))
}

/// Google crawlers and fetchers: https://developers.google.com/search/docs/crawling-indexing/overview-google-crawlers
/// Bing crawlers: https://www.bing.com/webmasters/help/which-crawlers-does-bing-use-8c184ec0
/// Bot user agents are listed in fastn-core/bot_user_agents.txt
static BOT_USER_AGENTS_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| {
        let bot_user_agents = include_str!("../bot_user_agents.txt").to_lowercase();
        let bot_user_agents = bot_user_agents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<&str>>()
            .join("|");
        regex::Regex::new(&bot_user_agents).unwrap()
    });

/// Checks whether a request was made by a Google/Bing bot based on its User-Agent
pub fn is_bot(user_agent: &str) -> bool {
    BOT_USER_AGENTS_REGEX.is_match(&user_agent.to_ascii_lowercase())
}

#[test]
fn test_is_bot() {
    assert!(is_bot("Mozilla/5.0 AppleWebKit/537.36 (KHTML, like Gecko; compatible; bingbot/2.0; +http://www.bing.com/bingbot.htm) Chrome/"));
    assert!(is_bot("Mozilla/5.0 (Linux; Android 6.0.1; Nexus 5X Build/MMB29P) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/W.X.Y.Z Mobile Safari/537.36 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)) Chrome/"));
    assert!(!is_bot("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));
}

pub(crate) fn get_header_key(header_key: &str) -> Option<&str> {
    if let Some(remaining) = header_key.strip_prefix("$header-") {
        return remaining.strip_suffix('$');
    }

    None
}

#[cfg(test)]
mod test {
    use actix_web::body::MessageBody;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn user_err() -> fastn_core::Result<()> {
        let user_err = vec!["invalid email".into()];
        let token_err = vec!["no key expected with name token".into()];
        let errors = vec![
            ("user".into(), user_err.clone()),
            ("token".into(), token_err.clone()),
        ];

        let res = fastn_core::http::user_err(errors, fastn_core::http::StatusCode::BAD_REQUEST)?;

        assert_eq!(res.status(), fastn_core::http::StatusCode::BAD_REQUEST);

        #[derive(serde::Deserialize)]
        struct Errors {
            user: Vec<String>,
            token: Vec<String>,
        }

        #[derive(serde::Deserialize)]
        struct TestErrorResponse {
            success: bool,
            errors: Errors,
        }

        let bytes = res.into_body().try_into_bytes().unwrap();

        let body: TestErrorResponse = serde_json::from_slice(&bytes)?;

        assert_eq!(body.success, false);
        assert_eq!(body.errors.user, user_err);
        assert_eq!(body.errors.token, token_err);

        Ok(())
    }
}
