use ftd::interpreter::{ComponentExt, PropertyValueExt, ValueExt};

pub(crate) const TEST_FOLDER: &str = "_tests";
pub(crate) const FIXTURE_FOLDER: &str = "fixtures";
pub(crate) const TEST_FILE_EXTENSION: &str = ".test.ftd";
pub(crate) const FIXTURE_FILE_EXTENSION: &str = ".test.ftd";

// mandatory test parameters
pub(crate) const TEST_TITLE_HEADER: &str = "title";
pub(crate) const TEST_URL_HEADER: &str = "url";

// optional test parameters
pub(crate) const FIXTURE_HEADER: &str = "fixtures";
pub(crate) const TEST_ID_HEADER: &str = "id";
pub(crate) const QUERY_PARAMS_HEADER: &str = "query-params";
pub(crate) const QUERY_PARAMS_HEADER_KEY: &str = "key";
pub(crate) const QUERY_PARAMS_HEADER_VALUE: &str = "value";
pub(crate) const POST_BODY_HEADER: &str = "body";
pub(crate) const TEST_CONTENT_HEADER: &str = "test";
pub(crate) const HTTP_REDIRECT_HEADER: &str = "http-redirect";
pub(crate) const HTTP_STATUS_HEADER: &str = "http-status";
pub(crate) const HTTP_LOCATION_HEADER: &str = "http-location";

macro_rules! log_variable {
    // When verbose is true, debug variables
    ($verbose:expr, $($variable:expr),*) => {
        if $verbose {
            $(std::dbg!($variable);)*
        }
    };
}

macro_rules! log_message {
    // When verbose is true, print message
    ($verbose:expr, $($message:expr),*) => {
        if $verbose {
            $(std::println!($message);)*
        }
    };
}

#[derive(Debug, Clone)]
pub struct TestParameters {
    pub script: bool,
    pub verbose: bool,
    pub instruction_number: i64,
    pub test_results: ftd::Map<String>,
    pub test_data: ftd::Map<String>,
}

impl TestParameters {
    pub fn new(script: bool, verbose: bool) -> Self {
        TestParameters {
            script,
            verbose,
            instruction_number: 0,
            test_results: Default::default(),
            test_data: Default::default(),
        }
    }
}

pub async fn test(
    config: &fastn_core::Config,
    only_id: Option<&str>,
    _base_url: &str,
    headless: bool,
    script: bool,
    verbose: bool,
) -> fastn_core::Result<()> {
    use colored::Colorize;

    if !headless {
        return fastn_core::usage_error(
            "Currently headless mode is only supported, use: --headless flag".to_string(),
        );
    }
    let ftd_documents = config.get_test_files().await?;

    for document in ftd_documents {
        if let Some(id) = only_id {
            if !document.id.contains(id) {
                continue;
            }
        }
        let mut test_parameters = TestParameters::new(script, verbose);
        println!("Running test file: {}", document.id.magenta());
        read_ftd_test_file(document, config, &mut test_parameters).await?;
    }

    Ok(())
}

impl fastn_core::Config {
    /**
    Returns the list of all fixture files with extension of `<file name>.test.ftd`
    **/
    pub(crate) async fn get_fixture_files(&self) -> fastn_core::Result<Vec<fastn_core::Document>> {
        use itertools::Itertools;
        let package = &self.package;
        let path = self.get_root_for_package(package);
        let all_files = self.get_all_fixture_file_paths().await?;
        let documents =
            fastn_core::paths_to_files(&self.ds, package.name.as_str(), all_files, &path, &None)
                .await?;
        let mut fixtures = documents
            .into_iter()
            .filter_map(|file| match file {
                fastn_core::File::Ftd(ftd_document)
                    if ftd_document
                        .id
                        .ends_with(fastn_core::commands::test::FIXTURE_FILE_EXTENSION) =>
                {
                    Some(ftd_document)
                }
                _ => None,
            })
            .collect_vec();
        fixtures.sort_by_key(|v| v.id.to_string());

        Ok(fixtures)
    }

    pub(crate) async fn get_all_fixture_file_paths(
        &self,
    ) -> fastn_core::Result<Vec<fastn_ds::Path>> {
        let path = self
            .get_root_for_package(&self.package)
            .join(fastn_core::commands::test::TEST_FOLDER)
            .join(fastn_core::commands::test::FIXTURE_FOLDER);
        Ok(self.ds.get_all_file_path(&path, &[]).await)
    }

    /**
    Returns the list of all test files with extension of `<file name>.test.ftd`
    **/
    pub(crate) async fn get_test_files(&self) -> fastn_core::Result<Vec<fastn_core::Document>> {
        use itertools::Itertools;

        let package = &self.package;
        let path = self.get_root_for_package(package);
        let all_files = self.get_all_test_file_paths().await?;
        let documents =
            fastn_core::paths_to_files(&self.ds, package.name.as_str(), all_files, &path, &None)
                .await?;
        let mut tests = documents
            .into_iter()
            .filter_map(|file| match file {
                fastn_core::File::Ftd(ftd_document)
                    if ftd_document
                        .id
                        .ends_with(fastn_core::commands::test::TEST_FILE_EXTENSION) =>
                {
                    Some(ftd_document)
                }
                _ => None,
            })
            .collect_vec();
        tests.sort_by_key(|v| v.id.to_string());

        Ok(tests)
    }

    pub(crate) async fn get_all_test_file_paths(&self) -> fastn_core::Result<Vec<fastn_ds::Path>> {
        let path = self
            .get_root_for_package(&self.package)
            .join(fastn_core::commands::test::TEST_FOLDER);
        let ignored_directories = ["fixtures".to_string()];
        Ok(self.ds.get_all_file_path(&path, &ignored_directories).await)
    }

    pub(crate) fn get_test_directory_path(&self) -> fastn_ds::Path {
        self.get_root_for_package(&self.package)
            .join(fastn_core::commands::test::TEST_FOLDER)
    }
}

#[async_recursion::async_recursion(? Send)]
async fn read_only_instructions(
    ftd_document: fastn_core::Document,
    config: &fastn_core::Config,
) -> fastn_core::Result<Vec<fastn_resolved::ComponentInvocation>> {
    let req = fastn_core::http::Request::default();
    let base_url = "/";
    let mut req_config =
        fastn_core::RequestConfig::new(config, &req, ftd_document.id.as_str(), base_url);
    req_config.current_document = Some(ftd_document.id.to_string());
    let main_ftd_doc = fastn_core::doc::interpret_helper(
        ftd_document.id_with_package().as_str(),
        ftd_document.content.as_str(),
        &mut req_config,
        base_url,
        false,
        0,
        &None,
    )
    .await?;

    let doc = ftd::interpreter::TDoc::new(
        &main_ftd_doc.name,
        &main_ftd_doc.aliases,
        &main_ftd_doc.data,
    );
    get_all_instructions(&main_ftd_doc.tree, &doc, config).await
}

async fn read_ftd_test_file(
    ftd_document: fastn_core::Document,
    config: &fastn_core::Config,
    test_parameters: &mut TestParameters,
) -> fastn_core::Result<()> {
    let req = fastn_core::http::Request::default();
    let mut saved_cookies: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let base_url = "/";
    let mut req_config =
        fastn_core::RequestConfig::new(config, &req, ftd_document.id.as_str(), base_url);
    req_config.current_document = Some(ftd_document.id.to_string());
    let main_ftd_doc = fastn_core::doc::interpret_helper(
        ftd_document.id_with_package().as_str(),
        ftd_document.content.as_str(),
        &mut req_config,
        base_url,
        false,
        0,
        &None,
    )
    .await?;

    let mut bag = main_ftd_doc.data.clone();
    bag.extend(ftd::interpreter::default::default_test_bag());

    let doc = ftd::interpreter::TDoc::new(&main_ftd_doc.name, &main_ftd_doc.aliases, &bag);
    let all_instructions = get_all_instructions(&main_ftd_doc.tree, &doc, config).await?;
    let mut instruction_number = 1;
    for instruction in all_instructions.iter() {
        test_parameters.instruction_number = instruction_number;
        if !execute_instruction(
            instruction,
            &doc,
            config,
            &mut saved_cookies,
            test_parameters,
        )
        .await?
        {
            break;
        }
        instruction_number += 1;
    }
    Ok(())
}

// This will give all overall set of instructions for a test file
// including instructions from fixture and other test instructions
async fn get_all_instructions(
    instructions: &[fastn_resolved::ComponentInvocation],
    doc: &ftd::interpreter::TDoc<'_>,
    config: &fastn_core::Config,
) -> fastn_core::Result<Vec<fastn_resolved::ComponentInvocation>> {
    let mut fixture_and_test_instructions = vec![];
    let mut rest_instructions = vec![];
    let mut included_fixtures: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut found_test_component = false;
    for instruction in instructions.iter() {
        match instruction.name.as_str() {
            "fastn#test" => {
                // Fixture instructions
                if found_test_component {
                    return fastn_core::usage_error(format!(
                        "'fastn.test' already exists, and another instance of it is not allowed \
                        in the same file., doc: {} line_number: {}",
                        doc.name, instruction.line_number
                    ));
                }

                found_test_component = true;
                fixture_and_test_instructions.extend(
                    get_instructions_from_test(instruction, doc, config, &mut included_fixtures)
                        .await?,
                );
            }
            "fastn#get" | "fastn#post" | "fastn#redirect" => {
                if !found_test_component {
                    return fastn_core::usage_error(format!(
                        "fastn.test doesn't exist for this test, doc: {} \
                        line_number: {}",
                        doc.name, instruction.line_number
                    ));
                }
                rest_instructions.push(instruction.clone())
            }
            t => {
                return fastn_core::usage_error(format!(
                    "Unknown instruction {}, line number: {}",
                    t, instruction.line_number
                ));
            }
        }
    }
    // instructions from fastn.test (fixture and fastn.test children instructions)
    let mut all_instructions = fixture_and_test_instructions;
    // Rest instructions if fastn.test not used at all
    all_instructions.extend(rest_instructions);

    Ok(all_instructions)
}

async fn execute_instruction(
    instruction: &fastn_resolved::ComponentInvocation,
    doc: &ftd::interpreter::TDoc<'_>,
    config: &fastn_core::Config,
    saved_cookies: &mut std::collections::HashMap<String, String>,
    test_parameters: &mut TestParameters,
) -> fastn_core::Result<bool> {
    match instruction.name.as_str() {
        "fastn#get" => {
            execute_get_instruction(instruction, doc, config, saved_cookies, test_parameters).await
        }
        "fastn#post" => {
            execute_post_instruction(instruction, doc, config, saved_cookies, test_parameters).await
        }
        "fastn#redirect" => {
            execute_redirect_instruction(instruction, doc, config, saved_cookies, test_parameters)
                .await
        }
        t => fastn_core::usage_error(format!(
            "Unknown instruction {}, line number: {}",
            t, instruction.line_number
        )),
    }
}

async fn get_instructions_from_test(
    instruction: &fastn_resolved::ComponentInvocation,
    doc: &ftd::interpreter::TDoc<'_>,
    config: &fastn_core::Config,
    included_fixtures: &mut std::collections::HashSet<String>,
) -> fastn_core::Result<Vec<fastn_resolved::ComponentInvocation>> {
    let property_values = instruction.get_interpreter_property_value_of_all_arguments(doc)?;

    if let Some(title) = get_optional_value_string(TEST_TITLE_HEADER, &property_values, doc)? {
        println!("Test: {}", title);
    }

    let fixtures =
        if let Some(fixtures) = get_optional_value_list(FIXTURE_HEADER, &property_values, doc)? {
            let mut resolved_fixtures = vec![];
            for fixture in fixtures.iter() {
                if let fastn_resolved::Value::String { text } = fixture {
                    resolved_fixtures.push(text.to_string());
                }
            }
            resolved_fixtures
        } else {
            vec![]
        };

    let fixture_instructions =
        get_fixture_instructions(config, fixtures, included_fixtures).await?;

    let all_instructions = fixture_instructions;
    Ok(all_instructions)
}

async fn get_fixture_instructions(
    config: &fastn_core::Config,
    fixtures: Vec<String>,
    included_fixtures: &mut std::collections::HashSet<String>,
) -> fastn_core::Result<Vec<fastn_resolved::ComponentInvocation>> {
    let mut fixture_instructions = vec![];

    for fixture_file_name in fixtures.iter() {
        if !included_fixtures.contains(fixture_file_name.as_str()) {
            let instructions =
                read_fixture_instructions(config, fixture_file_name.as_str()).await?;
            fixture_instructions.extend(instructions);
            included_fixtures.insert(fixture_file_name.to_string());
        }
    }

    Ok(fixture_instructions)
}

async fn read_fixture_instructions(
    config: &fastn_core::Config,
    fixture_file_name: &str,
) -> fastn_core::Result<Vec<fastn_resolved::ComponentInvocation>> {
    let fixture_files = config.get_fixture_files().await?;
    let current_fixture_file = fixture_files.iter().find(|d| {
        d.id.trim_start_matches(format!("{}/{}/", TEST_FOLDER, FIXTURE_FOLDER).as_str())
            .trim_end_matches(FIXTURE_FILE_EXTENSION)
            .eq(fixture_file_name)
    });

    if current_fixture_file.is_none() {
        return fastn_core::usage_error(format!(
            "Fixture: {} not found inside fixtures folder",
            fixture_file_name
        ));
    }

    read_only_instructions(current_fixture_file.unwrap().clone(), config).await
}

async fn execute_post_instruction(
    instruction: &fastn_resolved::ComponentInvocation,
    doc: &ftd::interpreter::TDoc<'_>,
    config: &fastn_core::Config,
    saved_cookies: &mut std::collections::HashMap<String, String>,
    test_parameters: &mut TestParameters,
) -> fastn_core::Result<bool> {
    let property_values = instruction.get_interpreter_property_value_of_all_arguments(doc)?;

    // Mandatory test parameters --------------------------------
    let url = get_value_ok(TEST_URL_HEADER, &property_values, instruction.line_number)?
        .to_json_string(doc, false)?
        .unwrap();
    let title = get_value_ok(TEST_TITLE_HEADER, &property_values, instruction.line_number)?
        .to_json_string(doc, false)?
        .unwrap();

    // Optional test parameters --------------------------------
    let mut optional_params: ftd::Map<String> = ftd::Map::new();

    if let Some(test_id) = get_optional_value_string(TEST_ID_HEADER, &property_values, doc)? {
        optional_params.insert(TEST_ID_HEADER.to_string(), test_id);
    }

    if let Some(test_content) =
        get_optional_value_string(TEST_CONTENT_HEADER, &property_values, doc)?
    {
        optional_params.insert(TEST_CONTENT_HEADER.to_string(), test_content);
    }

    if let Some(post_body) = get_optional_value_string(POST_BODY_HEADER, &property_values, doc)? {
        optional_params.insert(POST_BODY_HEADER.to_string(), post_body);
    }

    if let Some(http_status) = get_optional_value_string(HTTP_STATUS_HEADER, &property_values, doc)?
    {
        optional_params.insert(HTTP_STATUS_HEADER.to_string(), http_status);
    }

    if let Some(http_location) =
        get_optional_value_string(HTTP_LOCATION_HEADER, &property_values, doc)?
    {
        optional_params.insert(HTTP_LOCATION_HEADER.to_string(), http_location);
    }

    if let Some(http_redirect) =
        get_optional_value_string(HTTP_REDIRECT_HEADER, &property_values, doc)?
    {
        optional_params.insert(HTTP_REDIRECT_HEADER.to_string(), http_redirect);
    }

    assert_optional_headers(&optional_params)?;

    get_post_response_for_id(
        url.as_str(),
        title.as_str(),
        optional_params,
        config,
        saved_cookies,
        doc.name,
        test_parameters,
    )
    .await
}

async fn get_post_response_for_id(
    id: &str,
    title: &str,
    optional_params: ftd::Map<String>,
    config: &fastn_core::Config,
    saved_cookies: &mut std::collections::HashMap<String, String>,
    doc_name: &str,
    test_parameters: &mut TestParameters,
) -> fastn_core::Result<bool> {
    use actix_web::body::MessageBody;
    use colored::Colorize;

    println!("Test: {}", title.yellow());
    log_message!(test_parameters.verbose, "Test type: GET");
    log_variable!(test_parameters.verbose, &test_parameters.script);

    let req_body = optional_params
        .get(POST_BODY_HEADER)
        .cloned()
        .unwrap_or_default();

    let post_body = actix_web::web::Bytes::copy_from_slice(req_body.as_bytes());

    let actix_request = actix_web::test::TestRequest::with_uri(id)
        .method(actix_web::http::Method::POST)
        .insert_header(actix_web::http::header::ContentType::json())
        .to_http_request();

    let mut request = fastn_core::http::Request::from_actix(actix_request, post_body);

    request.set_cookies(saved_cookies);

    log_message!(test_parameters.verbose, "Request details");
    log_variable!(test_parameters.verbose, &request);

    let response = fastn_core::commands::serve::actix_serve(config, request, true, &None)
        .await?
        .0;
    update_cookies(saved_cookies, &response);

    let test_data = fastn_test_data(&response, test_parameters);

    log_message!(test_parameters.verbose, "Response details");
    log_variable!(test_parameters.verbose, &response);

    let (response_status_code, response_location) = assert_response(&response, &optional_params)?;
    let response_content_type = get_content_type(&response).unwrap_or("text/html".to_string());
    let test = optional_params.get(TEST_CONTENT_HEADER);
    if let Some(test_content) = test {
        let body = response.into_body().try_into_bytes().unwrap(); // Todo: Throw error
        let just_response_body = std::str::from_utf8(&body).unwrap();
        let response_js_data = if response_content_type.eq("application/json") {
            // Save Test results
            test_parameters.test_results.insert(
                test_parameters.instruction_number.to_string(),
                just_response_body.to_string(),
            );
            // Save Test result at its id key as well (if given)
            if let Some(test_id) = optional_params.get(TEST_ID_HEADER) {
                test_parameters
                    .test_results
                    .insert(test_id.clone(), just_response_body.to_string());
            }
            format!("fastn.http_response = {}", just_response_body)
        } else {
            // considering raw text when json response is not received
            format!("fastn.http_response = \"{}\";", just_response_body.trim())
        };

        log_message!(test_parameters.verbose, "fastn.http_response = ");
        log_variable!(test_parameters.verbose, &response_js_data);

        // Previous Test results variable
        let test_results_variable = if test_parameters.test_results.is_empty() {
            "".to_string()
        } else {
            make_test_results_variable(&test_parameters.test_results)
        };

        log_message!(test_parameters.verbose, "Previous Test results");
        log_variable!(test_parameters.verbose, &test_results_variable);

        // Todo: Throw error
        let fastn_test_js = fastn_js::fastn_test_js();
        let fastn_assertion_headers =
            fastn_js::fastn_assertion_headers(response_status_code, response_location.as_str());
        let fastn_js = fastn_js::all_js_without_test_and_ftd_langugage_js();

        let test_string = format!(
            "{fastn_js}\n{test_data}\n{response_js_data}\n{test_results_variable}\n\
                {fastn_assertion_headers}\n{fastn_test_js}\n{test_content}\
                \nfastn.test_result"
        );

        if test_parameters.script {
            let mut test_file_name = doc_name.to_string();
            if let Some((_, file_name)) = test_file_name.trim_end_matches('/').rsplit_once('/') {
                test_file_name = file_name.to_string();
            }
            generate_script_file(
                test_string.as_str(),
                &config.get_test_directory_path(),
                test_file_name
                    .replace(
                        ".test",
                        format!(".t{}.test", test_parameters.instruction_number).as_str(),
                    )
                    .as_str(),
                &config.ds,
            )
            .await;
            println!("{}", "Script file created".green());
            return Ok(true);
        }

        let test_result = fastn_js::run_test(test_string.as_str())?;

        if test_result.iter().any(|v| !(*v)) {
            println!("{}", "Test Failed".red());
            return Ok(false);
        }
    }
    println!("{}", "Test Passed".green());
    Ok(true)
}

async fn execute_get_instruction(
    instruction: &fastn_resolved::ComponentInvocation,
    doc: &ftd::interpreter::TDoc<'_>,
    config: &fastn_core::Config,
    saved_cookies: &mut std::collections::HashMap<String, String>,
    test_parameters: &mut TestParameters,
) -> fastn_core::Result<bool> {
    let property_values = instruction.get_interpreter_property_value_of_all_arguments(doc)?;

    // Mandatory test parameters --------------------------------
    let url = get_value_ok(TEST_URL_HEADER, &property_values, instruction.line_number)?
        .to_json_string(doc, false)?
        .unwrap();
    let title = get_value_ok(TEST_TITLE_HEADER, &property_values, instruction.line_number)?
        .to_json_string(doc, false)?
        .unwrap();

    // Optional test parameters --------------------------------
    let mut optional_params: ftd::Map<String> = ftd::Map::new();

    if let Some(test_id) = get_optional_value_string(TEST_ID_HEADER, &property_values, doc)? {
        optional_params.insert(TEST_ID_HEADER.to_string(), test_id);
    }

    if let Some(query_params) = get_optional_value_list(QUERY_PARAMS_HEADER, &property_values, doc)?
    {
        let mut query_strings = vec![];
        for query in query_params.iter() {
            if let fastn_resolved::Value::Record { fields, .. } = query {
                let resolved_key = fields
                    .get(QUERY_PARAMS_HEADER_KEY)
                    .unwrap()
                    .clone()
                    .resolve(doc, 0)?
                    .to_json_string(doc, false)?
                    .unwrap();
                let resolved_value = fields
                    .get(QUERY_PARAMS_HEADER_VALUE)
                    .unwrap()
                    .clone()
                    .resolve(doc, 0)?
                    .to_json_string(doc, false)?
                    .unwrap();
                let query_key_value =
                    format!("{}={}", resolved_key.as_str(), resolved_value.as_str());
                query_strings.push(query_key_value);
            }
        }
        if !query_strings.is_empty() {
            let query_string = query_strings.join("&").to_string();
            optional_params.insert(QUERY_PARAMS_HEADER.to_string(), query_string);
        }
    }

    if let Some(test_content) =
        get_optional_value_string(TEST_CONTENT_HEADER, &property_values, doc)?
    {
        optional_params.insert(TEST_CONTENT_HEADER.to_string(), test_content);
    }

    if let Some(http_status) = get_optional_value_string(HTTP_STATUS_HEADER, &property_values, doc)?
    {
        optional_params.insert(HTTP_STATUS_HEADER.to_string(), http_status);
    }

    if let Some(http_location) =
        get_optional_value_string(HTTP_LOCATION_HEADER, &property_values, doc)?
    {
        optional_params.insert(HTTP_LOCATION_HEADER.to_string(), http_location);
    }

    if let Some(http_redirect) =
        get_optional_value_string(HTTP_REDIRECT_HEADER, &property_values, doc)?
    {
        optional_params.insert(HTTP_REDIRECT_HEADER.to_string(), http_redirect);
    }

    assert_optional_headers(&optional_params)?;

    get_js_for_id(
        url.as_str(),
        title.as_str(),
        optional_params,
        config,
        saved_cookies,
        doc.name,
        test_parameters,
    )
    .await
}

fn get_content_type(response: &actix_web::HttpResponse) -> Option<String> {
    response
        .headers()
        .get(actix_web::http::header::CONTENT_TYPE)
        .and_then(|content_type| content_type.to_str().ok().map(String::from))
}

async fn get_js_for_id(
    id: &str,
    title: &str,
    optional_params: ftd::Map<String>,
    config: &fastn_core::Config,
    saved_cookies: &mut std::collections::HashMap<String, String>,
    doc_name: &str,
    test_parameters: &mut TestParameters,
) -> fastn_core::Result<bool> {
    use actix_web::body::MessageBody;
    use colored::Colorize;

    println!("Test: {}", title.yellow());
    log_message!(test_parameters.verbose, "Test type: GET");
    log_variable!(test_parameters.verbose, &test_parameters.script);

    let mut request = fastn_core::http::Request::default();
    request.path = id.to_string();
    if let Some(query_string) = optional_params.get(QUERY_PARAMS_HEADER) {
        request.set_query_string(query_string.as_str());
    }
    request.set_method("get");
    request.set_cookies(saved_cookies);

    log_message!(test_parameters.verbose, "Request details");
    log_variable!(test_parameters.verbose, &request);

    let response = fastn_core::commands::serve::actix_serve(config, request, true, &None)
        .await?
        .0;
    update_cookies(saved_cookies, &response);

    let test_data = fastn_test_data(&response, test_parameters);

    log_message!(test_parameters.verbose, "Response details");
    log_variable!(test_parameters.verbose, &response);

    let (response_status_code, response_location) = assert_response(&response, &optional_params)?;
    let response_content_type = get_content_type(&response).unwrap_or("text/html".to_string());
    let test = optional_params.get(TEST_CONTENT_HEADER);
    if let Some(test_content) = test {
        let body = response.into_body().try_into_bytes().unwrap(); // Todo: Throw error
        let just_response_body = std::str::from_utf8(&body).unwrap();
        let response_js_data = if response_content_type.eq("application/json") {
            // Save Test results
            test_parameters.test_results.insert(
                test_parameters.instruction_number.to_string(),
                just_response_body.to_string(),
            );
            // Save Test result at its id key as well (if given)
            if let Some(test_id) = optional_params.get(TEST_ID_HEADER) {
                test_parameters
                    .test_results
                    .insert(test_id.clone(), just_response_body.to_string());
            }
            format!("fastn.http_response = {}", just_response_body)
        } else {
            just_response_body.to_string()
        };

        // Previous Test results variable
        let test_results_variable = if test_parameters.test_results.is_empty() {
            "".to_string()
        } else {
            make_test_results_variable(&test_parameters.test_results)
        };

        log_message!(test_parameters.verbose, "Previous Test results");
        log_variable!(test_parameters.verbose, &test_results_variable);

        let fastn_test_js = fastn_js::fastn_test_js();
        let fastn_assertion_headers =
            fastn_js::fastn_assertion_headers(response_status_code, response_location.as_str());
        let fastn_js = fastn_js::all_js_without_test_and_ftd_langugage_js();
        let test_string = format!(
            "{fastn_js}\n{test_data}\n{response_js_data}\n{test_results_variable}\n\
                {fastn_assertion_headers}\n{fastn_test_js}\n{test_content}\
                \nfastn.test_result"
        );
        if test_parameters.script {
            let mut test_file_name = doc_name.to_string();
            if let Some((_, file_name)) = test_file_name.trim_end_matches('/').rsplit_once('/') {
                test_file_name = file_name.to_string();
            }
            generate_script_file(
                test_string.as_str(),
                &config.get_test_directory_path(),
                test_file_name
                    .replace(
                        ".test",
                        format!(".t{}.test", test_parameters.instruction_number).as_str(),
                    )
                    .as_str(),
                &config.ds,
            )
            .await;
            println!("{}", "Script file created".green());
            return Ok(true);
        }
        let test_result = fastn_js::run_test(test_string.as_str())?;
        if test_result.iter().any(|v| !(*v)) {
            println!("{}", "Test Failed".red());
            return Ok(false);
        }
    }
    println!("{}", "Test Passed".green());
    Ok(true)
}

fn make_test_results_variable(test_results: &ftd::Map<String>) -> String {
    let mut test_results_variable = "fastn.test_results = {};\n".to_string();
    for (key, value) in test_results.iter() {
        test_results_variable.push_str(
            format!(
                "fastn.test_results[\"{}\"] = {};\n",
                key.as_str(),
                value.as_str()
            )
            .as_str(),
        )
    }
    test_results_variable
}

fn update_cookies(
    saved_cookies: &mut std::collections::HashMap<String, String>,
    response: &actix_web::HttpResponse,
) {
    for ref c in response.cookies() {
        saved_cookies.insert(c.name().to_string(), c.value().to_string());
    }
}

fn get_value_ok(
    key: &str,
    property_values: &ftd::Map<fastn_resolved::PropertyValue>,
    line_number: usize,
) -> fastn_core::Result<fastn_resolved::Value> {
    get_value(key, property_values).ok_or(fastn_core::Error::NotFound(format!(
        "Key '{}' not found, line number: {}",
        key, line_number
    )))
}

fn get_value(
    key: &str,
    property_values: &ftd::Map<fastn_resolved::PropertyValue>,
) -> Option<fastn_resolved::Value> {
    let property_value = property_values.get(key)?;
    match property_value {
        fastn_resolved::PropertyValue::Value { value, .. } => Some(value.clone()),
        _ => unimplemented!(),
    }
}

fn get_optional_value(
    key: &str,
    property_values: &ftd::Map<fastn_resolved::PropertyValue>,
) -> Option<fastn_resolved::Value> {
    if let Some(property_value) = property_values.get(key) {
        return match property_value {
            fastn_resolved::PropertyValue::Value { value, .. } => Some(value.clone()),
            _ => unimplemented!(),
        };
    }
    None
}

fn get_optional_value_list(
    key: &str,
    property_values: &ftd::Map<fastn_resolved::PropertyValue>,
    doc: &ftd::interpreter::TDoc<'_>,
) -> ftd::interpreter::Result<Option<Vec<fastn_resolved::Value>>> {
    let value = get_optional_value(key, property_values);
    if let Some(ref value) = value {
        return value.to_list(doc, false);
    }
    Ok(None)
}

fn get_optional_value_string(
    key: &str,
    property_values: &ftd::Map<fastn_resolved::PropertyValue>,
    doc: &ftd::interpreter::TDoc<'_>,
) -> ftd::interpreter::Result<Option<String>> {
    let value = get_optional_value(key, property_values);
    if let Some(ref value) = value {
        return value.to_json_string(doc, false);
    }
    Ok(None)
}

pub fn test_fastn_ftd() -> &'static str {
    include_str!("../../test_fastn.ftd")
}

pub fn assert_optional_headers(
    optional_test_parameters: &ftd::Map<String>,
) -> fastn_core::Result<bool> {
    if (optional_test_parameters.contains_key(HTTP_STATUS_HEADER)
        || optional_test_parameters.contains_key(HTTP_LOCATION_HEADER))
        && optional_test_parameters.contains_key(HTTP_REDIRECT_HEADER)
    {
        return fastn_core::usage_error(format!(
            "Use either [{} and {}] or [{}] both not allowed.",
            HTTP_STATUS_HEADER, HTTP_LOCATION_HEADER, HTTP_REDIRECT_HEADER
        ));
    }
    Ok(true)
}

pub fn assert_response(
    response: &actix_web::HttpResponse,
    params: &ftd::Map<String>,
) -> fastn_core::Result<(u16, String)> {
    if let Some(redirection_url) = params.get(HTTP_REDIRECT_HEADER) {
        return assert_redirect(response, redirection_url);
    }

    assert_location_and_status(response, params)
}

pub fn assert_redirect(
    response: &actix_web::HttpResponse,
    redirection_location: &str,
) -> fastn_core::Result<(u16, String)> {
    let response_status_code = response.status().as_u16();
    if !response.status().is_redirection() {
        return fastn_core::assert_error(format!(
            "Invalid redirect status code {:?}",
            response.status().as_u16()
        ));
    }

    let response_location = get_response_location(response)?.unwrap_or_default();
    if !response_location.eq(redirection_location) {
        return fastn_core::assert_error(format!(
            "HTTP redirect location mismatch. Expected \"{:?}\", Found \"{:?}\"",
            redirection_location, response_location
        ));
    }

    Ok((response_status_code, response_location))
}

pub fn assert_location_and_status(
    response: &actix_web::HttpResponse,
    params: &ftd::Map<String>,
) -> fastn_core::Result<(u16, String)> {
    // By default, we are expecting status 200 if not http-status is not passed
    let default_status_code = "200".to_string();
    let response_status_code = response.status().as_u16();
    let response_status_code_string = response_status_code.to_string();
    let expected_status_code = params
        .get(HTTP_STATUS_HEADER)
        .unwrap_or(&default_status_code);

    if !response_status_code_string.eq(expected_status_code) {
        return fastn_core::assert_error(format!(
            "HTTP status code mismatch. Expected {}, Found {}",
            expected_status_code, response_status_code
        ));
    }

    let response_location = get_response_location(response)?.unwrap_or_default();
    let expected_location = params.get(HTTP_LOCATION_HEADER);

    if let Some(expected_location) = expected_location {
        if !expected_location.eq(response_location.as_str()) {
            return fastn_core::assert_error(format!(
                "HTTP Location mismatch. Expected \"{:?}\", Found \"{:?}\"",
                expected_location, response_location
            ));
        }
    }

    Ok((response_status_code, response_location))
}

pub fn get_response_location(
    response: &actix_web::HttpResponse,
) -> fastn_core::Result<Option<String>> {
    if let Some(redirect_location) = response.headers().get("Location") {
        return if let Ok(location) = redirect_location.to_str() {
            Ok(Some(location.to_string()))
        } else {
            fastn_core::generic_error("Failed to convert 'Location' header to string".to_string())
        };
    }
    Ok(None)
}

async fn generate_script_file(
    content: &str,
    test_directory: &fastn_ds::Path,
    test_file_name: &str,
    ds: &fastn_ds::DocumentStore,
) {
    let html_content = format!(
        indoc::indoc! {"
                        <html>
                        <script>
                        {content}
                        </script>
                        </html>
                    "},
        content = content
    );
    let file_location = test_directory.join(test_file_name.replace(".test", ".script.html"));
    ds.write_content(&file_location, &html_content.into_bytes())
        .await
        .unwrap();
}

/// Extract test data from response headers
/// persists them across tests in `test_parameters.test_data`
fn fastn_test_data(
    response: &actix_web::HttpResponse,
    test_parameters: &mut TestParameters,
) -> String {
    use itertools::Itertools;

    let mut res = response
        .headers()
        .iter()
        .filter_map(|(k, v)| {
            if k.as_str().starts_with("x-fastn-test-") {
                let key = k
                    .as_str()
                    .strip_prefix("x-fastn-test-")
                    .unwrap()
                    .to_lowercase()
                    .replace('-', "_");

                let val = v.to_str().unwrap();

                test_parameters
                    .test_data
                    .insert(key.clone(), val.to_string());

                Some(format!("fastn.test_data[\"{}\"] = \"{}\";", key, val,))
            } else {
                None
            }
        })
        .join("\n");

    let existing_test_data = test_parameters
        .test_data
        .iter()
        .map(|(k, v)| format!("fastn.test_data[\"{}\"] = \"{}\";", k, v,))
        .join("\n");

    res.push_str(existing_test_data.as_str());
    res.insert_str(0, "fastn.test_data = {};\n");

    res
}

async fn execute_redirect_instruction(
    instruction: &fastn_resolved::ComponentInvocation,
    doc: &ftd::interpreter::TDoc<'_>,
    config: &fastn_core::Config,
    saved_cookies: &mut std::collections::HashMap<String, String>,
    test_parameters: &mut TestParameters,
) -> fastn_core::Result<bool> {
    let property_values = instruction.get_interpreter_property_value_of_all_arguments(doc)?;

    let redirect = get_value_ok(
        HTTP_REDIRECT_HEADER,
        &property_values,
        instruction.line_number,
    )?
    .to_json_string(doc, false)?
    .unwrap();

    let (redirect_from_url, redirect_to_url) = match redirect.split_once("->") {
        Some((from, to)) => (from.trim(), to.trim()),
        None => {
            return fastn_core::usage_error(
                "Invalid redirection format. Please use '->' to indicate the redirection URL."
                    .to_string(),
            );
        }
    };

    let mut params: ftd::Map<String> = ftd::Map::new();

    params.insert(
        HTTP_REDIRECT_HEADER.to_string(),
        redirect_to_url.to_string(),
    );

    get_js_for_id(
        redirect_from_url,
        format!(
            "Redirecting from {} -> {}",
            redirect_from_url, redirect_to_url,
        )
        .as_str(),
        params,
        config,
        saved_cookies,
        doc.name,
        test_parameters,
    )
    .await
}
