pub trait ValueOf {
    fn value_of_(&self, name: &str) -> Option<&str>;
    fn values_of_(&self, name: &str) -> Vec<String>;
}

impl ValueOf for clap::ArgMatches {
    fn value_of_(&self, name: &str) -> Option<&str> {
        self.get_one::<String>(name).map(|v| v.as_str())
    }
    fn values_of_(&self, name: &str) -> Vec<String> {
        self.get_many(name)
            .map(|v| v.cloned().collect::<Vec<String>>())
            .unwrap_or_default()
    }
}

// https://stackoverflow.com/questions/71985357/whats-the-best-way-to-write-a-custom-format-macro
#[macro_export]
macro_rules! warning {
    ($($t:tt)*) => {{
        use colored::Colorize;
        let msg = format!($($t)*);
        if fastn_observer::is_traced() {
            tracing::warn!(msg);
        } else {
            eprintln!("WARN: {}", msg.yellow());
        }
        msg
    }};
}

fn id_to_cache_key(id: &str) -> String {
    // TODO: use MAIN_SEPARATOR here
    id.replace(['/', '\\'], "_")
}

pub fn get_ftd_hash(path: &str) -> fastn_core::Result<String> {
    let path = fastn_core::utils::replace_last_n(path, 1, "/", "");
    Ok(fastn_core::utils::generate_hash(
        std::fs::read(format!("{path}.ftd"))
            .or_else(|_| std::fs::read(format!("{path}/index.ftd")))?,
    ))
}

pub fn get_cache_file(id: &str) -> Option<std::path::PathBuf> {
    let cache_dir = dirs::cache_dir()?;
    let base_path = cache_dir.join("fastn.com");

    if !base_path.exists() {
        if let Err(err) = std::fs::create_dir_all(&base_path) {
            eprintln!("Failed to create cache directory: {}", err);
            return None;
        }
    }

    Some(
        base_path
            .join(id_to_cache_key(
                &std::env::current_dir()
                    .expect("cant read current dir")
                    .to_string_lossy(),
            ))
            .join(id_to_cache_key(id)),
    )
}

pub fn get_cached<T>(id: &str) -> Option<T>
where
    T: serde::de::DeserializeOwned,
{
    let cache_file = get_cache_file(id)?;
    serde_json::from_str(
        std::fs::read_to_string(cache_file)
            .inspect_err(|e| tracing::debug!("file read error: {}", e.to_string()))
            .ok()?
            .as_str(),
    )
    .inspect_err(|e| tracing::debug!("not valid json: {}", e.to_string()))
    .ok()
}

pub fn cache_it<T>(id: &str, d: T) -> ftd::interpreter::Result<T>
where
    T: serde::ser::Serialize,
{
    let cache_file = get_cache_file(id)
        .ok_or_else(|| ftd::interpreter::Error::OtherError("cache dir not found".to_string()))?;
    std::fs::create_dir_all(cache_file.parent().unwrap()).map_err(|e| {
        ftd::interpreter::Error::OtherError(format!("failed to create cache dir: {}", e))
    })?;
    std::fs::write(cache_file, serde_json::to_string(&d)?).map_err(|e| {
        ftd::interpreter::Error::OtherError(format!("failed to write cache file: {}", e))
    })?;
    Ok(d)
}

pub fn redirect_page_html(url: &str) -> String {
    include_str!("../redirect.html").replace("__REDIRECT_URL__", url)
}

pub fn print_end(msg: &str, start: std::time::Instant) {
    use colored::Colorize;

    if fastn_core::utils::is_test() {
        println!("done in <omitted>");
    } else {
        println!(
            // TODO: instead of lots of spaces put proper erase current terminal line thing
            "\r{:?} {} in {:?}.                          ",
            std::time::Instant::now(),
            msg.green(),
            start.elapsed()
        );
    }
}

/// replace_last_n("a.b.c.d.e.f", 2, ".", "/") => "a.b.c.d/e/f"
pub fn replace_last_n(s: &str, n: usize, pattern: &str, replacement: &str) -> String {
    use itertools::Itertools;

    s.rsplitn(n + 1, pattern)
        .collect_vec()
        .into_iter()
        .rev()
        .join(replacement)
}

#[cfg(test)]
mod test {
    #[test]
    fn is_static_path() {
        assert!(super::is_static_path("/foo/bar.js"));
        assert!(super::is_static_path("/bar.js"));
        assert!(!super::is_static_path("/foo/bar.js/"));
        assert!(!super::is_static_path("/bar.js/"));
        assert!(!super::is_static_path("/foo/bar.ftd"));
        assert!(!super::is_static_path("/foo/bar.ftd/"));
        assert!(!super::is_static_path("/foo/bar"));
        assert!(!super::is_static_path("/foo/bar/"));
    }

    #[test]
    fn replace_last_n() {
        assert_eq!(
            super::replace_last_n("a.b.c.d.e.f", 2, ".", "/"),
            "a.b.c.d/e/f"
        );
        assert_eq!(
            super::replace_last_n("a.b.c.d.e.", 2, ".", "/"),
            "a.b.c.d/e/"
        );
        assert_eq!(super::replace_last_n("d-e.f", 2, ".", "/"), "d-e/f");
        assert_eq!(
            super::replace_last_n("a.ftd/b.ftd", 1, ".ftd", "/index.html"),
            "a.ftd/b/index.html"
        );
        assert_eq!(
            super::replace_last_n("index.ftd/b/index.ftd", 1, "index.ftd", "index.html"),
            "index.ftd/b/index.html"
        );
    }
}

pub fn print_error(msg: &str, start: std::time::Instant) {
    use colored::Colorize;

    if fastn_core::utils::is_test() {
        println!("done in <omitted>");
    } else {
        eprintln!(
            "\r{:?} {} in {:?}.                          ",
            std::time::Instant::now(),
            msg.red(),
            start.elapsed(),
        );
    }
}

pub fn value_to_colored_string(value: &serde_json::Value, indent_level: u32) -> String {
    use colored::Colorize;

    match value {
        serde_json::Value::Null => "null".bright_black().to_string(),
        serde_json::Value::Bool(v) => v.to_string().bright_green().to_string(),
        serde_json::Value::Number(v) => v.to_string().bright_blue().to_string(),
        serde_json::Value::String(v) => format!(
            "\"{}\"",
            v.replace('\\', "\\\\")
                .replace('\n', "\\n")
                .replace('\"', "\\\"")
        )
        .bright_yellow()
        .to_string(),
        serde_json::Value::Array(v) => {
            let mut s = String::new();
            for (idx, value) in v.iter().enumerate() {
                s.push_str(&format!(
                    "{comma}\n{indent}{value}",
                    indent = "  ".repeat(indent_level as usize),
                    value = value_to_colored_string(value, indent_level + 1),
                    comma = if idx.eq(&0) { "" } else { "," }
                ));
            }
            format!("[{}\n{}]", s, "  ".repeat((indent_level - 1) as usize))
        }
        serde_json::Value::Object(v) => {
            let mut s = String::new();
            for (idx, (key, value)) in v.iter().enumerate() {
                s.push_str(&format!(
                    "{comma}\n{indent}\"{i}\": {value}",
                    indent = "  ".repeat(indent_level as usize),
                    i = key.bright_cyan(),
                    value = value_to_colored_string(value, indent_level + 1),
                    comma = if idx.eq(&0) { "" } else { "," }
                ));
            }
            format!("{{{}\n{}}}", s, "  ".repeat((indent_level - 1) as usize))
        }
    }
}

pub fn value_to_colored_string_without_null(
    value: &serde_json::Value,
    indent_level: u32,
) -> String {
    use colored::Colorize;

    match value {
        serde_json::Value::Null => "".to_string(),
        serde_json::Value::Bool(v) => v.to_string().bright_green().to_string(),
        serde_json::Value::Number(v) => v.to_string().bright_blue().to_string(),
        serde_json::Value::String(v) => format!(
            "\"{}\"",
            v.replace('\\', "\\\\")
                .replace('\n', "\\n")
                .replace('\"', "\\\"")
        )
        .bright_yellow()
        .to_string(),
        serde_json::Value::Array(v) if v.is_empty() => "".to_string(),
        serde_json::Value::Array(v) => {
            let mut s = String::new();
            let mut is_first = true;
            for value in v.iter() {
                let value_string = value_to_colored_string_without_null(value, indent_level + 1);
                if !value_string.is_empty() {
                    s.push_str(&format!(
                        "{comma}\n{indent}{value}",
                        indent = "  ".repeat(indent_level as usize),
                        value = value_string,
                        comma = if is_first { "" } else { "," }
                    ));
                    is_first = false;
                }
            }
            if s.is_empty() {
                "".to_string()
            } else {
                format!("[{}\n{}]", s, "  ".repeat((indent_level - 1) as usize))
            }
        }
        serde_json::Value::Object(v) => {
            let mut s = String::new();
            let mut is_first = true;
            for (key, value) in v {
                let value_string = value_to_colored_string_without_null(value, indent_level + 1);
                if !value_string.is_empty() {
                    s.push_str(&format!(
                        "{comma}\n{indent}\"{i}\": {value}",
                        indent = "  ".repeat(indent_level as usize),
                        i = key.bright_cyan(),
                        value = value_string,
                        comma = if is_first { "" } else { "," }
                    ));
                    is_first = false;
                }
            }
            format!("{{{}\n{}}}", s, "  ".repeat((indent_level - 1) as usize))
        }
    }
}

pub fn time(msg: &str) -> Timer {
    Timer {
        start: std::time::Instant::now(),
        msg,
    }
}

pub struct Timer<'a> {
    start: std::time::Instant,
    msg: &'a str,
}

impl<'a> Timer<'a> {
    pub fn it<T>(&self, a: T) -> T {
        use colored::Colorize;

        if !fastn_core::utils::is_test() {
            let duration = format!("{:?}", self.start.elapsed());
            println!("{} in {}", self.msg.green(), duration.red());
        }

        a
    }
}

pub trait HasElements {
    fn has_elements(&self) -> bool;
}

impl<T> HasElements for Vec<T> {
    fn has_elements(&self) -> bool {
        !self.is_empty()
    }
}

pub(crate) fn history_path(
    id: &str,
    base_path: &fastn_ds::Path,
    timestamp: &u128,
) -> fastn_ds::Path {
    let id_with_timestamp_extension = snapshot_id(id, timestamp);
    base_path.join(".history").join(id_with_timestamp_extension)
}

pub(crate) fn snapshot_id(path: &str, timestamp: &u128) -> String {
    if let Some((id, ext)) = path.rsplit_once('.') {
        format!("{}.{}.{}", id, timestamp, ext)
    } else {
        format!("{}.{}", path, timestamp)
    }
}

pub(crate) fn track_path(id: &str, base_path: &fastn_ds::Path) -> fastn_ds::Path {
    base_path.join(".tracks").join(format!("{}.track", id))
}

pub(crate) async fn get_number_of_documents(
    _config: &fastn_core::Config,
) -> fastn_core::Result<String> {
    Ok(0.to_string())
}

pub(crate) async fn get_last_modified_on(
    _ds: &fastn_ds::DocumentStore,
    _path: &fastn_ds::Path,
) -> Option<String> {
    None
}

/*
// todo get_package_title needs to be implemented
    @amitu need to come up with idea
    This data would be used in fastn.title
pub(crate) fn get_package_title(config: &fastn_core::Config) -> String {
    let fastn = if let Ok(fastn) = std::fs::read_to_string(config.ds.root().join("index.ftd")) {
        fastn
    } else {
        return config.package.name.clone();
    };
    let lib = fastn_core::Library {
        config: config.clone(),
        markdown: None,
        document_id: "index.ftd".to_string(),
        translated_data: Default::default(),
        current_package: std::sync::Arc::new(std::sync::Mutex::new(vec![config.package.clone()])),
    };
    let main_ftd_doc = match ftd::p2::Document::from("index.ftd", fastn.as_str(), &lib) {
        Ok(v) => v,
        Err(_) => {
            return config.package.name.clone();
        }
    };
    match &main_ftd_doc.title() {
        Some(x) => x.rendered.clone(),
        _ => config.package.name.clone(),
    }
}*/

/*#[async_recursion::async_recursion(?Send)]
pub async fn copy_dir_all(
    src: impl AsRef<camino::Utf8Path> + 'static,
    dst: impl AsRef<camino::Utf8Path> + 'static,
) -> std::io::Result<()> {
    tokio::fs::create_dir_all(dst.as_ref()).await?;
    let mut dir = tokio::fs::read_dir(src.as_ref()).await?;
    while let Some(child) = dir.next_entry().await? {
        if child.metadata().await?.is_dir() {
            copy_dir_all(
                fastn_ds::Path::from_path_buf(child.path()).expect("we only work with utf8 paths"),
                dst.as_ref().join(
                    child
                        .file_name()
                        .into_string()
                        .expect("we only work with utf8 paths"),
                ),
            )
            .await?;
        } else {
            tokio::fs::copy(
                child.path(),
                dst.as_ref().join(
                    child
                        .file_name()
                        .into_string()
                        .expect("we only work with utf8 paths"),
                ),
            )
            .await?;
        }
    }
    Ok(())
}*/

pub(crate) fn validate_base_url(package: &fastn_core::Package) -> fastn_core::Result<()> {
    if package.download_base_url.is_none() {
        warning!("expected base in fastn.package: {:?}", package.name);
    }

    Ok(())
}

pub fn escape_string(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '\"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\0' => result.push_str("\\0"),
            _ => result.push(c),
        }
    }
    result
}

#[allow(dead_code)]
pub fn escape_ftd(file: &str) -> String {
    use itertools::Itertools;

    file.split('\n')
        .map(|v| {
            if v.starts_with("-- ") || v.starts_with("--- ") {
                format!("\\{}", v)
            } else {
                v.to_string()
            }
        })
        .join("\n")
}

pub fn id_to_path(id: &str) -> String {
    id.replace("/index.ftd", "/")
        .replace("index.ftd", "/")
        .replace(".ftd", std::path::MAIN_SEPARATOR.to_string().as_str())
        .replace("/index.md", "/")
        .replace("/README.md", "/")
        .replace("index.md", "/")
        .replace("README.md", "/")
        .replace(".md", std::path::MAIN_SEPARATOR.to_string().as_str())
}

/// returns true if an existing file named "file_name"
/// exists in the root package folder
async fn is_file_in_root(
    root: &str,
    file_name: &str,
    ds: &fastn_ds::DocumentStore,
    session_id: &Option<String>,
) -> bool {
    ds.exists(&fastn_ds::Path::new(root).join(file_name), session_id)
        .await
}

/// returns favicon html tag as string
/// (if favicon is passed as header in fastn.package or if any favicon.* file is present in the root package folder)
/// otherwise returns None
async fn resolve_favicon(
    root_path: &str,
    package_name: &str,
    favicon: &Option<String>,
    ds: &fastn_ds::DocumentStore,
    session_id: &Option<String>,
) -> Option<String> {
    /// returns html tag for using favicon.
    fn favicon_html(favicon_path: &str, content_type: &str) -> String {
        let favicon_html = format!(
            "\n<link rel=\"shortcut icon\" href=\"{}\" type=\"{}\">",
            favicon_path, content_type
        );
        favicon_html
    }

    /// returns relative favicon path from package and its mime content type
    fn get_favicon_path_and_type(package_name: &str, favicon_path: &str) -> (String, String) {
        // relative favicon path wrt package
        let path = fastn_ds::Path::new(package_name).join(favicon_path);
        // mime content type of the favicon
        let content_type = mime_guess::from_path(path.to_string().as_str()).first_or_octet_stream();

        (favicon_path.to_string(), content_type.to_string())
    }

    // favicon image path from fastn.package if provided
    let fav_path = favicon;

    let (full_fav_path, fav_mime_content_type): (String, String) = {
        match fav_path {
            Some(ref path) => {
                // In this case, favicon is provided with fastn.package in FASTN.ftd
                get_favicon_path_and_type(package_name, path)
            }
            None => {
                // If favicon not provided so we will look for favicon in the package directory
                // By default if any file favicon.* is present we will use that file instead
                // In case of favicon.* conflict priority will be: .ico > .svg > .png > .jpg.
                // Default searching directory being the root folder of the package

                // Just check if any favicon exists in the root package directory
                // in the above mentioned priority order
                let found_favicon_id =
                    if is_file_in_root(root_path, "favicon.ico", ds, session_id).await {
                        "favicon.ico"
                    } else if is_file_in_root(root_path, "favicon.svg", ds, session_id).await {
                        "favicon.svg"
                    } else if is_file_in_root(root_path, "favicon.png", ds, session_id).await {
                        "favicon.png"
                    } else if is_file_in_root(root_path, "favicon.jpg", ds, session_id).await {
                        "favicon.jpg"
                    } else {
                        // Not using any favicon
                        return None;
                    };

                get_favicon_path_and_type(package_name, found_favicon_id)
            }
        }
    };

    // Will use some favicon
    Some(favicon_html(&full_fav_path, &fav_mime_content_type))
}

pub fn get_external_js_html(external_js: &[String]) -> String {
    let mut result = "".to_string();
    for js in external_js {
        result = format!("{}<script src=\"{}\"></script>", result, js);
    }
    result
}

pub fn get_external_css_html(external_js: &[String]) -> String {
    let mut result = "".to_string();
    for js in external_js {
        result = format!("{}<link rel=\"stylesheet\" href=\"{}.css\">", result, js);
    }
    result
}

pub async fn get_inline_js_html(
    config: &fastn_core::Config,
    inline_js: &[String],
    session_id: &Option<String>,
) -> String {
    let mut result = "".to_string();
    for path in inline_js {
        let path = fastn_ds::Path::new(path);
        if let Ok(content) = config.ds.read_to_string(&path, session_id).await {
            result = format!("{}<script>{}</script>", result, content);
        }
    }
    result
}

pub async fn get_inline_css_html(
    config: &fastn_core::Config,
    inline_js: &[String],
    session_id: &Option<String>,
) -> String {
    let mut result = "".to_string();
    for path in inline_js {
        let path = fastn_ds::Path::new(path);
        if let Ok(content) = config.ds.read_to_string(&path, session_id).await {
            result = format!("{}<style>{}</style>", result, content);
        }
    }
    result
}

async fn get_extra_js(
    config: &fastn_core::Config,
    external_js: &[String],
    inline_js: &[String],
    js: &str,
    rive_data: &str,
    session_id: &Option<String>,
) -> String {
    format!(
        "{}{}{}{}",
        get_external_js_html(external_js),
        get_inline_js_html(config, inline_js, session_id).await,
        js,
        rive_data
    )
}

async fn get_extra_css(
    config: &fastn_core::Config,
    external_css: &[String],
    inline_css: &[String],
    css: &str,
    session_id: &Option<String>,
) -> String {
    format!(
        "{}{}{}",
        get_external_css_html(external_css),
        get_inline_css_html(config, inline_css, session_id).await,
        css
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn replace_markers_2022(
    s: &str,
    html_ui: ftd::html::HtmlUI,
    config: &fastn_core::Config,
    main_id: &str,
    font_style: &str,
    base_url: &str,
    session_id: &Option<String>,
) -> String {
    ftd::html::utils::trim_all_lines(
        s.replace(
            "__ftd_meta_data__",
            ftd::html::utils::get_meta_data(&html_ui.html_data).as_str(),
        )
        .replace(
            "__ftd_doc_title__",
            html_ui.html_data.title.unwrap_or_default().as_str(),
        )
        .replace("__ftd_data__", html_ui.variables.as_str())
        .replace(
            "__ftd_canonical_url__",
            config.package.generate_canonical_url(main_id).as_str(),
        )
        .replace(
            "__favicon_html_tag__",
            resolve_favicon(
                config.ds.root().to_string().as_str(),
                config.package.name.as_str(),
                &config.package.favicon,
                &config.ds,
                session_id,
            )
            .await
            .unwrap_or_default()
            .as_str(),
        )
        .replace("__ftd_external_children__", "{}")
        .replace("__hashed_default_css__", hashed_default_css_name())
        .replace("__hashed_default_js__", hashed_default_js_name())
        .replace(
            "__ftd__",
            format!("{}{}", html_ui.html.as_str(), font_style).as_str(),
        )
        .replace(
            "__extra_js__",
            get_extra_js(
                config,
                config.ftd_external_js.as_slice(),
                config.ftd_inline_js.as_slice(),
                html_ui.js.as_str(),
                html_ui.rive_data.as_str(),
                session_id,
            )
            .await
            .as_str(),
        )
        .replace(
            "__extra_css__",
            get_extra_css(
                config,
                config.ftd_external_css.as_slice(),
                config.ftd_inline_css.as_slice(),
                html_ui.css.as_str(),
                session_id,
            )
            .await
            .as_str(),
        )
        .replace(
            "__ftd_functions__",
            format!(
                "{}\n{}\n{}\n{}\n{}\n{}\n{}",
                html_ui.functions.as_str(),
                html_ui.dependencies.as_str(),
                html_ui.variable_dependencies.as_str(),
                html_ui.dummy_html.as_str(),
                html_ui.raw_html.as_str(),
                html_ui.mutable_variable,
                html_ui.immutable_variable
            )
            .as_str(),
        )
        .replace("__ftd_body_events__", html_ui.outer_events.as_str())
        .replace("__ftd_element_css__", "")
        .replace("__base_url__", base_url)
        .as_str(),
    )
}

pub fn get_fastn_package_data(package: &fastn_core::Package) -> String {
    format!(
        indoc::indoc! {"
        let __fastn_package_name__ = \"{package_name}\";
    "},
        package_name = package.name
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn replace_markers_2023(
    js_script: &str,
    scripts: &str,
    ssr_body: &str,
    font_style: &str,
    default_css: &str,
    base_url: &str,
    config: &fastn_core::Config,
    session_id: &Option<String>,
) -> String {
    format!(
        include_str!("../../ftd/ftd-js.html"),
        fastn_package = get_fastn_package_data(&config.package).as_str(),
        base_url_tag = if !base_url.is_empty() {
            format!("<base href=\"{}\">", base_url)
        } else {
            "".to_string()
        },
        favicon_html_tag = resolve_favicon(
            config.ds.root().to_string().as_str(),
            config.package.name.as_str(),
            &config.package.favicon,
            &config.ds,
            session_id,
        )
        .await
        .unwrap_or_default()
        .as_str(),
        js_script = format!("{js_script}{}", fastn_core::utils::available_code_themes()).as_str(),
        script_file = format!(
            r#"
                <script src="{}"></script>
                <script src="{}"></script>
                <link rel="stylesheet" href="{}">
                {}
            "#,
            hashed_prism_js(),
            hashed_default_ftd_js(config.package.name.as_str()),
            hashed_prism_css(),
            scripts,
        )
        .as_str(),
        extra_js = get_extra_js(
            config,
            config.ftd_external_js.as_slice(),
            config.ftd_inline_js.as_slice(),
            "",
            "",
            session_id,
        )
        .await
        .as_str(),
        default_css = default_css,
        html_body = format!("{}{}", ssr_body, font_style).as_str(),
    )
}

pub fn is_test() -> bool {
    cfg!(test) || std::env::args().any(|e| e == "--test")
}

pub(crate) async fn write(
    root: &fastn_ds::Path,
    file_path: &str,
    data: &[u8],
    ds: &fastn_ds::DocumentStore,
    session_id: &Option<String>,
) -> fastn_core::Result<()> {
    if ds.exists(&root.join(file_path), session_id).await {
        return Ok(());
    }
    update1(root, file_path, data, ds).await
}

pub(crate) async fn overwrite(
    root: &fastn_ds::Path,
    file_path: &str,
    data: &[u8],
    ds: &fastn_ds::DocumentStore,
) -> fastn_core::Result<()> {
    update1(root, file_path, data, ds).await
}

// TODO: remove this function use update instead
pub async fn update1(
    root: &fastn_ds::Path,
    file_path: &str,
    data: &[u8],
    ds: &fastn_ds::DocumentStore,
) -> fastn_core::Result<()> {
    let (file_root, file_name) = if let Some((file_root, file_name)) = file_path.rsplit_once('/') {
        (file_root.to_string(), file_name.to_string())
    } else {
        ("".to_string(), file_path.to_string())
    };

    Ok(ds
        .write_content(&root.join(file_root).join(file_name), data)
        .await?)
}

pub(crate) async fn copy(
    from: &fastn_ds::Path,
    to: &fastn_ds::Path,
    ds: &fastn_ds::DocumentStore,
) -> fastn_core::Result<()> {
    let content = ds.read_content(from, &None).await?;
    fastn_core::utils::update(to, content.as_slice(), ds).await
}

pub async fn update(
    root: &fastn_ds::Path,
    data: &[u8],
    ds: &fastn_ds::DocumentStore,
) -> fastn_core::Result<()> {
    let (file_root, file_name) = if let Some(file_root) = root.parent() {
        (
            file_root,
            root.file_name()
                .ok_or_else(|| fastn_core::Error::UsageError {
                    message: format!("Invalid File Path: Can't find file name `{:?}`", root),
                })?,
        )
    } else {
        return Err(fastn_core::Error::UsageError {
            message: format!(
                "Invalid File Path: file path doesn't have parent: {:?}",
                root
            ),
        });
    };

    Ok(ds.write_content(&file_root.join(file_name), data).await?)
}

pub(crate) fn ids_matches(id1: &str, id2: &str) -> bool {
    return strip_id(id1).eq(&strip_id(id2));

    fn strip_id(id: &str) -> String {
        let id = id
            .trim()
            .replace("/index.html", "/")
            .replace("index.html", "/");
        if id.eq("/") {
            return id;
        }
        id.trim_matches('/').to_string()
    }
}

/// Parse argument from CLI
/// If CLI command: fastn serve --identities a@foo.com,foo
/// key: --identities -> output: a@foo.com,foo
pub fn parse_from_cli(key: &str) -> Option<String> {
    use itertools::Itertools;
    let args = std::env::args().collect_vec();
    let mut index = None;
    for (idx, arg) in args.iter().enumerate() {
        if arg.eq(key) {
            index = Some(idx);
        }
    }
    index
        .and_then(|idx| args.get(idx + 1))
        .map(String::to_string)
}

/// Remove from provided `root` except given list
pub async fn remove_except(
    root: &fastn_ds::Path,
    except: &[&str],
    ds: &fastn_ds::DocumentStore,
) -> fastn_core::Result<()> {
    use itertools::Itertools;
    let except = except.iter().map(|x| root.join(x)).collect_vec();
    for path in ds.get_all_file_path(root, &[]).await {
        if except.contains(&path) {
            continue;
        }
        ds.remove(&path).await?;
    }
    Ok(())
}

/// /api/?a=1&b=2&c=3 => vec[(a, 1), (b, 2), (c, 3)]
pub fn query(uri: &str) -> fastn_core::Result<Vec<(String, String)>> {
    use itertools::Itertools;
    Ok(
        url::Url::parse(format!("https://fifthtry.com/{}", uri).as_str())?
            .query_pairs()
            .into_owned()
            .collect_vec(),
    )
}

pub fn generate_hash(content: impl AsRef<[u8]>) -> String {
    use sha2::digest::FixedOutput;
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(content);
    format!("{:X}", hasher.finalize_fixed())
}

static CSS_HASH: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| format!("default-{}.css", generate_hash(ftd::css())));

pub fn hashed_default_css_name() -> &'static str {
    &CSS_HASH
}

static JS_HASH: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
    format!(
        "default-{}.js",
        generate_hash(format!("{}\n\n{}", ftd::build_js(), fastn_core::fastn_2022_js()).as_str())
    )
});

pub fn hashed_default_js_name() -> &'static str {
    &JS_HASH
}

static FTD_JS_HASH: once_cell::sync::OnceCell<String> = once_cell::sync::OnceCell::new();

pub fn hashed_default_ftd_js(package_name: &str) -> &'static str {
    FTD_JS_HASH.get_or_init(|| {
        format!(
            "default-{}.js",
            generate_hash(ftd::js::all_js_without_test(package_name).as_str())
        )
    })
}

static MARKDOWN_HASH: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| format!("markdown-{}.js", generate_hash(ftd::markdown_js()),));

pub fn hashed_markdown_js() -> &'static str {
    &MARKDOWN_HASH
}

static PRISM_JS_HASH: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| format!("prism-{}.js", generate_hash(ftd::prism_js().as_str()),));

pub fn hashed_prism_js() -> &'static str {
    &PRISM_JS_HASH
}

static PRISM_CSS_HASH: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
    format!("prism-{}.css", generate_hash(ftd::prism_css().as_str()),)
});

pub fn hashed_prism_css() -> &'static str {
    &PRISM_CSS_HASH
}

static CODE_THEME_HASH: once_cell::sync::Lazy<ftd::Map<String>> =
    once_cell::sync::Lazy::new(|| {
        ftd::theme_css()
            .into_iter()
            .map(|(k, v)| (k, format!("code-theme-{}.css", generate_hash(v.as_str()))))
            .collect()
    });

pub fn hashed_code_theme_css() -> &'static ftd::Map<String> {
    &CODE_THEME_HASH
}

pub fn available_code_themes() -> String {
    let themes = hashed_code_theme_css();
    let mut result = vec![];
    for (theme, url) in themes {
        result.push(format!(
            "fastn_dom.codeData.availableThemes[\"{theme}\"] = \"{url}\";"
        ))
    }
    result.join("\n")
}

#[cfg(test)]
mod tests {
    #[test]
    fn query() {
        assert_eq!(
            super::query("/api/?a=1&b=2&c=3").unwrap(),
            vec![
                ("a".to_string(), "1".to_string()),
                ("b".to_string(), "2".to_string()),
                ("c".to_string(), "3".to_string()),
            ]
        )
    }
}

pub fn ignore_headers() -> Vec<&'static str> {
    vec!["host", "x-forwarded-ssl"]
}

pub(crate) fn is_static_path(path: &str) -> bool {
    assert!(path.starts_with('/'));

    if path.starts_with("/ide/") {
        // temporary hack
        return false;
    }

    match path
        .rsplit_once('/')
        .map(|(_, k)| k)
        .and_then(|k| k.rsplit_once('.').map(|(_, ext)| ext))
    {
        Some("ftd") => false,
        Some(_) => true,
        None => false,
    }
}

static VARIABLE_INTERPOLATION_RGX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"\$\{([^}]+)\}").unwrap());

pub(crate) async fn interpolate_env_vars(
    ds: &fastn_ds::DocumentStore,
    endpoint: &str,
) -> fastn_core::Result<String> {
    let mut result = String::new();
    let mut last_end = 0;

    for captures in VARIABLE_INTERPOLATION_RGX.captures_iter(endpoint) {
        let capture = captures.get(0).unwrap();
        let start = capture.start();
        let end = capture.end();
        result.push_str(&endpoint[last_end..start]);

        let key = captures.get(1).unwrap().as_str().trim();

        let value = match get_interpolated_value(ds, key).await {
            Ok(value) => value,
            Err(e) => {
                return fastn_core::generic_error(format!(
                    "Failed to interpolate value in endpoint: {e}"
                ));
            }
        };

        result.push_str(&value);

        last_end = end;
    }

    result.push_str(&endpoint[last_end..]);
    Ok(result)
}

async fn get_interpolated_value(
    ds: &fastn_ds::DocumentStore,
    input: &str,
) -> fastn_core::Result<String> {
    let value = match fastn_expr::interpolator::get_var_name_and_default(input)? {
        (Some(var_name), default_value) => match var_name {
            key if key.starts_with("env.") => {
                let env_key = key.trim_start_matches("env.");

                get_env_value_or_default(ds, env_key, default_value).await?
            }
            _ => {
                return Err(fastn_core::error::Error::generic(format!(
                    "unknown variable '{}'.",
                    input,
                )));
            }
        },
        (None, Some(default_value)) => default_value,
        _ => {
            return Err(fastn_core::error::Error::generic(
                "unsupported interpolation syntax used.".to_string(),
            ));
        }
    };

    Ok(value)
}

async fn get_env_value_or_default(
    ds: &fastn_ds::DocumentStore,
    env_key: &str,
    default_value: Option<String>,
) -> fastn_core::Result<String> {
    match ds.env(env_key).await {
        Ok(value) => Ok(value),
        Err(e) => {
            if let Some(default_value) = default_value {
                Ok(default_value)
            } else {
                Err(fastn_core::error::Error::generic(format!(
                    "could not find environment variable '{}': {e}",
                    env_key
                )))
            }
        }
    }
}

pub async fn secret_key(ds: &fastn_ds::DocumentStore) -> String {
    match ds.env("FASTN_SECRET_KEY").await {
        Ok(secret) => secret,
        Err(_e) => {
            fastn_core::warning!(
                "WARN: Using default SECRET_KEY. Provide one using FASTN_SECRET_KEY env var."
            );
            "FASTN_TEMP_SECRET".to_string()
        }
    }
}

pub fn fifthtry_site_zip_url(site_slug: &str) -> String {
    format!("https://fifthtry.com/ft2/api/site/download?site-slug={site_slug}",)
}
