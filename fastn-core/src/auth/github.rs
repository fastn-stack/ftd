#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserDetail {
    pub access_token: String,
    pub username: String,
}

pub async fn login(
    req: &fastn_core::http::Request,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    let redirect_url: String = format!(
        "{}://{}/-/auth/github/?next={}",
        req.connection_info.scheme(),
        req.connection_info.host(),
        next, // TODO: we should url escape this
    );

    // Note: public_repos user:email all these things are github resources
    // So we have to tell oauth_client who is getting logged in what are we going to access
    let (mut authorize_url, _token) = utils::github_client()
        .set_redirect_uri(oauth2::RedirectUrl::new(redirect_url)?)
        .authorize_url(oauth2::CsrfToken::new_random)
        .add_scope(oauth2::Scope::new("public_repo".to_string()))
        .add_scope(oauth2::Scope::new("user:email".to_string()))
        .add_scope(oauth2::Scope::new("read:org".to_string()))
        .url();

    // https://openid.net/specs/openid-connect-core-1_0.html#AuthRequest:~:text=an%20appropriate%20display.-,prompt,-OPTIONAL.%20Space%20delimited
    authorize_url
        .query_pairs_mut()
        .append_pair("prompt", "consent");

    Ok(fastn_core::http::redirect(authorize_url.to_string()))
}

// route: /-/auth/github/done/
// In this API we are accessing
// the token and setting it to cookies
pub async fn callback(
    req: &fastn_core::http::Request,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    let code = req.q("code", "".to_string())?;
    // TODO: CSRF check

    let access_token = match utils::github_client()
        .exchange_code(oauth2::AuthorizationCode::new(code))
        .request_async(oauth2::reqwest::async_http_client)
        .await
    {
        Ok(access_token) => oauth2::TokenResponse::access_token(&access_token)
            .secret()
            .to_string(),
        Err(e) => return Ok(fastn_core::server_error!("{}", e.to_string())),
    };

    let ud = UserDetail {
        username: apis::username(access_token.as_str()).await?,
        access_token,
    };

    let user_detail_str = serde_json::to_string(&ud)?;
    return Ok(actix_web::HttpResponse::Found()
        .cookie(
            actix_web::cookie::Cookie::build(
                fastn_core::auth::AuthProviders::GitHub.as_str(),
                fastn_core::auth::utils::encrypt_str(&user_detail_str).await,
            )
            .domain(fastn_core::auth::utils::domain(req.connection_info.host()))
            .path("/")
            .permanent()
            // TODO: AbrarK is running on http,
            // will remove it later
            // .secure(true)
            .finish(),
        )
        .append_header((actix_web::http::header::LOCATION, next))
        .finish());
}

// it returns identities which matches to given input
pub async fn matched_identities(
    ud: UserDetail,
    identities: &[fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    let github_identities = identities
        .iter()
        .filter(|identity| identity.key.starts_with("github"))
        .collect::<Vec<&fastn_core::user_group::UserIdentity>>();

    if github_identities.is_empty() {
        return Ok(vec![]);
    }

    let mut matched_identities = vec![];
    // matched_starred_repositories
    matched_identities.extend(matched_starred_repos(&ud, github_identities.as_slice()).await?);
    // matched: github-watches
    matched_identities.extend(matched_watched_repos(&ud, github_identities.as_slice()).await?);
    // matched: github-follows
    matched_identities.extend(matched_followed_org(&ud, github_identities.as_slice()).await?);
    // matched: github-contributor
    matched_identities.extend(matched_contributed_repos(&ud, github_identities.as_slice()).await?);
    // matched: github-collaborator
    matched_identities.extend(matched_collaborated_repos(&ud, github_identities.as_slice()).await?);
    // matched: github-team
    matched_identities.extend(matched_org_teams(&ud, github_identities.as_slice()).await?);
    // matched: github-sponsor
    matched_identities.extend(matched_sponsored_org(&ud, github_identities.as_slice()).await?);

    Ok(matched_identities)
}

pub async fn matched_starred_repos(
    ud: &UserDetail,
    identities: &[&fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    use itertools::Itertools;

    let starred_repos = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("github-starred") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();

    if starred_repos.is_empty() {
        return Ok(vec![]);
    }
    let user_starred_repos = apis::starred_repo(ud.access_token.as_str()).await?;
    // filter the user starred repos with input
    Ok(user_starred_repos
        .into_iter()
        .filter(|user_repo| starred_repos.contains(&user_repo.as_str()))
        .map(|repo| fastn_core::user_group::UserIdentity {
            key: "github-starred".to_string(),
            value: repo,
        })
        .collect())
}

pub async fn matched_watched_repos(
    ud: &UserDetail,
    identities: &[&fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    use itertools::Itertools;
    let watched_repos = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("github-watches") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();
    if watched_repos.is_empty() {
        return Ok(vec![]);
    }
    let user_watched_repos = apis::watched_repo(ud.access_token.as_str()).await?;
    // filter the user watched repos with input
    Ok(user_watched_repos
        .into_iter()
        .filter(|user_repo| watched_repos.contains(&user_repo.as_str()))
        .map(|repo| fastn_core::user_group::UserIdentity {
            key: "github-watches".to_string(),
            value: repo,
        })
        .collect())
}

pub async fn matched_followed_org(
    ud: &UserDetail,
    identities: &[&fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    use itertools::Itertools;
    let followed_orgs = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("github-follows") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();
    if followed_orgs.is_empty() {
        return Ok(vec![]);
    }
    let user_followed_orgs = apis::followed_org(ud.access_token.as_str()).await?;
    // filter the user followed orgs with input
    Ok(user_followed_orgs
        .into_iter()
        .filter(|user_org| followed_orgs.contains(&user_org.as_str()))
        .map(|org| fastn_core::user_group::UserIdentity {
            key: "github-follows".to_string(),
            value: org,
        })
        .collect())
}

pub async fn matched_contributed_repos(
    ud: &UserDetail,
    identities: &[&fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut matched_repo_contributors_list: Vec<String> = vec![];
    let contributed_repos = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("github-contributor") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();

    if contributed_repos.is_empty() {
        return Ok(vec![]);
    }
    for repo in &contributed_repos {
        let repo_contributors = apis::repo_contributors(ud.access_token.as_str(), repo).await?;

        if repo_contributors.contains(&ud.username) {
            matched_repo_contributors_list.push(String::from(repo.to_owned()));
        }
    }
    // filter the user contributed repos with input
    Ok(matched_repo_contributors_list
        .into_iter()
        .filter(|user_repo| contributed_repos.contains(&user_repo.as_str()))
        .map(|repo| fastn_core::user_group::UserIdentity {
            key: "github-contributor".to_string(),
            value: repo,
        })
        .collect())
}

pub async fn matched_collaborated_repos(
    ud: &UserDetail,
    identities: &[&fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut matched_repo_collaborator_list: Vec<String> = vec![];
    let collaborated_repos = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("github-collaborator") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();

    if collaborated_repos.is_empty() {
        return Ok(vec![]);
    }
    for repo in &collaborated_repos {
        let repo_collaborator = apis::repo_collaborators(ud.access_token.as_str(), repo).await?;

        if repo_collaborator.contains(&ud.username) {
            matched_repo_collaborator_list.push(String::from(repo.to_owned()));
        }
    }
    // filter the user collaborated repos with input
    Ok(matched_repo_collaborator_list
        .into_iter()
        .filter(|user_repo| collaborated_repos.contains(&user_repo.as_str()))
        .map(|repo| fastn_core::user_group::UserIdentity {
            key: "github-collaborator".to_string(),
            value: repo,
        })
        .collect())
}

pub async fn matched_org_teams(
    ud: &UserDetail,
    identities: &[&fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut matched_org_teams: Vec<String> = vec![];
    let org_teams = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("github-team") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();

    if org_teams.is_empty() {
        return Ok(vec![]);
    }

    for org_team in org_teams.iter() {
        if let Some((org_name, team_name)) = org_team.split_once('/') {
            let team_members: Vec<String> =
                apis::team_members(ud.access_token.as_str(), org_name, team_name).await?;
            if team_members.contains(&ud.username) {
                matched_org_teams.push(org_team.to_string());
            }
        }
        // TODO:
        // Return Error if org-name/team-name does not come
    }
    // filter the user joined teams with input
    Ok(matched_org_teams
        .into_iter()
        .map(|org_team| fastn_core::user_group::UserIdentity {
            key: "github-team".to_string(),
            value: org_team,
        })
        .collect())
}

pub async fn matched_sponsored_org(
    ud: &UserDetail,
    identities: &[&fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut sponsored_users_list: Vec<String> = vec![];

    let sponsors_list = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("github-sponsor") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();

    if sponsors_list.is_empty() {
        return Ok(vec![]);
    }

    for sponsor in sponsors_list.iter() {
        if apis::is_user_sponsored(
            ud.access_token.as_str(),
            ud.username.as_str(),
            sponsor.to_owned(),
        )
        .await?
        {
            sponsored_users_list.push(sponsor.to_string());
        }
    }
    // return the sponsor list
    Ok(sponsored_users_list
        .into_iter()
        .map(|sponsor| fastn_core::user_group::UserIdentity {
            key: "github-sponsor".to_string(),
            value: sponsor,
        })
        .collect())
}

pub mod apis {
    #[derive(Debug, serde::Deserialize)]
    pub struct GraphQLResp {
        pub data: Data,
    }

    #[derive(Debug, serde::Deserialize)]
    pub struct Data {
        pub user: User,
    }

    #[derive(Debug, serde::Deserialize)]
    pub struct User {
        #[serde(rename = "isSponsoredBy")]
        pub is_sponsored_by: bool,
    }

    // TODO: API to starred a repo on behalf of the user
    // API Docs: https://docs.github.com/en/rest/activity/starring#list-repositories-starred-by-the-authenticated-user
    pub async fn starred_repo(token: &str) -> fastn_core::Result<Vec<String>> {
        // API Docs: https://docs.github.com/en/rest/activity/starring#list-repositories-starred-by-the-authenticated-user
        // TODO: Handle paginated response

        #[derive(Debug, serde::Deserialize)]
        struct UserRepos {
            full_name: String,
        }
        let starred_repo: Vec<UserRepos> = fastn_core::auth::utils::get_api(
            "https://api.github.com/user/starred?per_page=100",
            token,
        )
        .await?;
        Ok(starred_repo.into_iter().map(|x| x.full_name).collect())
    }

    pub async fn followed_org(token: &str) -> fastn_core::Result<Vec<String>> {
        // API Docs: https://docs.github.com/en/rest/users/followers#list-followers-of-the-authenticated-user
        // TODO: Handle paginated response
        #[derive(Debug, serde::Deserialize)]
        struct FollowedOrg {
            login: String,
        }
        let watched_repo: Vec<FollowedOrg> = fastn_core::auth::utils::get_api(
            "https://api.github.com/user/following?per_page=100",
            token,
        )
        .await?;
        Ok(watched_repo.into_iter().map(|x| x.login).collect())
    }

    pub async fn team_members(
        token: &str,
        org_title: &str,
        team_slug: &str,
    ) -> fastn_core::Result<Vec<String>> {
        // API Docs: https://docs.github.com/en/rest/teams/members#list-team-members
        // TODO: Handle paginated response
        #[derive(Debug, serde::Deserialize)]
        struct TeamMembers {
            login: String,
        }

        let user_orgs: Vec<TeamMembers> = fastn_core::auth::utils::get_api(
            format!(
                "https://api.github.com/orgs/{org_title}/teams/{team_slug}/members?per_page=100",
            ),
            token,
        )
        .await?;
        Ok(user_orgs.into_iter().map(|x| x.login).collect())
    }

    pub async fn watched_repo(token: &str) -> fastn_core::Result<Vec<String>> {
        // API Docs: https://docs.github.com/en/rest/activity/watching#list-repositories-watched-by-the-authenticated-user
        // TODO: Handle paginated response
        #[derive(Debug, serde::Deserialize)]
        struct UserRepos {
            full_name: String,
        }
        let watched_repo: Vec<UserRepos> = fastn_core::auth::utils::get_api(
            "https://api.github.com/user/subscriptions?per_page=100",
            token,
        )
        .await?;
        Ok(watched_repo.into_iter().map(|x| x.full_name).collect())
    }

    pub async fn repo_contributors(
        token: &str,
        repo_name: &str,
    ) -> fastn_core::Result<Vec<String>> {
        // API Docs: https://docs.github.com/en/rest/activity/starring#list-repositories-starred-by-the-authenticated-user
        // TODO: Handle paginated response
        #[derive(Debug, serde::Deserialize)]
        struct RepoContributor {
            login: String,
        }
        let repo_contributor: Vec<RepoContributor> = fastn_core::auth::utils::get_api(
            format!("https://api.github.com/repos/{repo_name}/contributors?per_page=100",),
            token,
        )
        .await?;
        Ok(repo_contributor.into_iter().map(|x| x.login).collect())
    }

    pub async fn repo_collaborators(
        token: &str,
        repo_name: &str,
    ) -> fastn_core::Result<Vec<String>> {
        // API Docs: https://docs.github.com/en/rest/collaborators/collaborators#list-repository-collaborators
        // TODO: Handle paginated response
        #[derive(Debug, serde::Deserialize)]
        struct RepoCollaborator {
            login: String,
        }
        let repo_collaborators_list: Vec<RepoCollaborator> = fastn_core::auth::utils::get_api(
            format!("https://api.github.com/repos/{repo_name}/collaborators?per_page=100"),
            token,
        )
        .await?;
        Ok(repo_collaborators_list
            .into_iter()
            .map(|x| x.login)
            .collect())
    }

    pub async fn is_user_sponsored(
        token: &str,
        username: &str,
        sponsored_by: &str,
    ) -> fastn_core::Result<bool> {
        let query = format!(
            "query {{ user(login: \"{username}\") {{ isSponsoredBy(accountLogin: \"{sponsored_by}\" )}} }}",
        );
        let sponsor_obj =
            graphql_sponsor_api("https://api.github.com/graphql", query.as_str(), token).await?;
        if sponsor_obj.data.user.is_sponsored_by {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // TODO: It can be stored in the request cookies
    pub async fn username(access_token: &str) -> fastn_core::Result<String> {
        // API Docs: https://docs.github.com/en/rest/users/users#get-the-authenticated-user
        // TODO: Handle paginated response
        #[derive(Debug, serde::Deserialize)]
        struct UserDetails {
            login: String,
        }

        let user_obj: UserDetails =
            fastn_core::auth::utils::get_api("https://api.github.com/user", access_token).await?;

        Ok(String::from(&user_obj.login))
    }

    pub async fn graphql_sponsor_api(
        url: &str,
        query_str: &str,
        token: &str,
    ) -> fastn_core::Result<GraphQLResp> {
        let mut map: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();
        map.insert("query", query_str);

        let response = reqwest::Client::new()
            .post(url)
            .json(&map)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("{} {}", "Bearer", token),
            )
            .header(reqwest::header::ACCEPT, "application/json")
            .header(
                reqwest::header::USER_AGENT,
                reqwest::header::HeaderValue::from_static("fastn"),
            )
            .send()
            .await?;
        if !response.status().eq(&reqwest::StatusCode::OK) {
            return Err(fastn_core::Error::APIResponseError(format!(
                "GitHub API ERROR: {}",
                url
            )));
        }
        let return_obj = response.json::<GraphQLResp>().await?;

        Ok(return_obj)
    }
}

pub mod utils {
    // Lazy means a value which initialize at the first time access
    // we have to access it before using it and make sure to use it while starting a server
    // TODO: they should be configured with auth feature flag
    // if feature flag auth is enabled Make sure that before accessing in the API these variable
    // are set
    static GITHUB_CLIENT_ID: once_cell::sync::Lazy<oauth2::ClientId> = {
        once_cell::sync::Lazy::new(|| {
            oauth2::ClientId::new(match std::env::var("FASTN_GITHUB_CLIENT_ID") {
                Ok(val) => val,
                Err(e) => format!("{}{}", "FASTN_GITHUB_CLIENT_ID not set in env ", e),
            })
        })
    };

    static GITHUB_CLIENT_SECRET: once_cell::sync::Lazy<oauth2::ClientSecret> = {
        once_cell::sync::Lazy::new(|| {
            oauth2::ClientSecret::new(match std::env::var("FASTN_GITHUB_CLIENT_SECRET") {
                Ok(val) => val,
                Err(e) => format!("{}{}", "FASTN_GITHUB_CLIENT_SECRET not set in env ", e),
            })
        })
    };

    pub fn github_client() -> oauth2::basic::BasicClient {
        oauth2::basic::BasicClient::new(
            GITHUB_CLIENT_ID.to_owned(),
            Some(GITHUB_CLIENT_SECRET.to_owned()),
            oauth2::AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap(),
            Some(
                oauth2::TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
                    .expect("Invalid token endpoint URL"),
            ),
        )
    }
}
