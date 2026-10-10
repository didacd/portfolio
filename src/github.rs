use std::{
    collections::BTreeMap,
    io,
    time::{Duration, Instant},
};

use reqwest::{Client, redirect::Policy};
use serde::Deserialize;
use tokio::sync::Mutex;
use tracing::warn;

const CACHE_TTL: Duration = Duration::from_secs(600);

pub(crate) struct GithubCache {
    client: Client,
    cache: Mutex<Cache>,
}

#[derive(Default)]
struct Cache {
    username: String,
    attempted_at: Option<Instant>,
    data: Option<GithubData>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct GithubUser {
    pub(crate) public_repos: u64,
    pub(crate) followers: u64,
    pub(crate) following: u64,
    pub(crate) created_at: String,
}

#[derive(Deserialize)]
struct GithubRepo {
    fork: bool,
    stargazers_count: u64,
    language: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct GithubData {
    pub(crate) user: GithubUser,
    pub(crate) stars: u64,
    pub(crate) languages: Vec<String>,
}

impl GithubCache {
    pub(crate) fn new() -> io::Result<Self> {
        let client = Client::builder()
            .user_agent("didac-portfolio")
            .https_only(true)
            .redirect(Policy::none())
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(io::Error::other)?;
        Ok(Self {
            client,
            cache: Mutex::new(Cache::default()),
        })
    }

    pub(crate) async fn get(&self, username: &str) -> Option<GithubData> {
        if std::env::var("GITHUB_STATS").is_ok_and(|value| value == "off")
            || username.is_empty()
            || username.len() > 39
            || !username
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return None;
        }
        // One refresh at a time; failed requests are cached too to avoid rate-limit storms.
        let mut cache = self.cache.lock().await;
        if cache.username != username {
            *cache = Cache {
                username: username.to_owned(),
                ..Cache::default()
            };
        }
        if cache
            .attempted_at
            .is_some_and(|time| time.elapsed() < CACHE_TTL)
        {
            return cache.data.clone();
        }
        cache.attempted_at = Some(Instant::now());
        let user_path = format!("/users/{username}");
        let repos_path = format!("/users/{username}/repos?per_page=100");
        let (user, repos) = tokio::join!(
            self.fetch::<GithubUser>(&user_path),
            self.fetch::<Vec<GithubRepo>>(&repos_path),
        );
        if let Some(user) = user {
            let repos = repos.unwrap_or_default();
            let (stars, languages) = summarize_repos(&repos);
            cache.data = Some(GithubData {
                user,
                stars,
                languages,
            });
        }
        cache.data.clone()
    }

    async fn fetch<T: serde::de::DeserializeOwned>(&self, path: &str) -> Option<T> {
        let mut request = self
            .client
            .get(format!("https://api.github.com{path}"))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28");
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            request = request.bearer_auth(token);
        }
        let result = async { request.send().await?.error_for_status()?.json().await }.await;
        match result {
            Ok(data) => Some(data),
            Err(_) => {
                // Never print request headers or tokens. The page still renders without GitHub.
                warn!("GitHub profile unavailable; using cached data or local profile");
                None
            }
        }
    }
}

fn summarize_repos(repos: &[GithubRepo]) -> (u64, Vec<String>) {
    let mut stars = 0;
    let mut counts = BTreeMap::new();
    for repo in repos.iter().filter(|repo| !repo.fork) {
        stars += repo.stargazers_count;
        if let Some(language) = &repo.language {
            *counts.entry(language.clone()).or_insert(0usize) += 1;
        }
    }
    let mut languages: Vec<_> = counts.into_iter().collect();
    languages.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    (
        stars,
        languages
            .into_iter()
            .take(5)
            .map(|(name, _)| name)
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn github_totals_exclude_forks_and_rank_languages() {
        let repos: Vec<GithubRepo> = serde_json::from_str(
            r#"[
            {"fork":false,"stargazers_count":3,"language":"Rust"},
            {"fork":false,"stargazers_count":2,"language":"Rust"},
            {"fork":true,"stargazers_count":100,"language":"Python"}
        ]"#,
        )
        .unwrap();
        assert_eq!(summarize_repos(&repos), (5, vec!["Rust".to_owned()]));
    }
}
