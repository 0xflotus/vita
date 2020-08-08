use crate::error::{Error, Result};
use crate::IntoSubdomain;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

#[derive(Deserialize, Debug)]
struct Subdomain {
    hostname: String,
}

#[derive(Deserialize, Debug)]
struct AlienvaultResult {
    passive_dns: Vec<Subdomain>,
    count: i32,
}

impl IntoSubdomain for AlienvaultResult {
    fn subdomains(&self) -> HashSet<String> {
        self.passive_dns
            .iter()
            .map(|s| s.hostname.to_owned())
            .collect()
    }
}

fn build_url(host: &str) -> String {
    format!(
        "https://otx.alienvault.com/api/v1/indicators/domain/{}/passive_dns",
        host
    )
}

pub async fn run(host: Arc<String>) -> Result<HashSet<String>> {
    trace!("fetching data from alienvault for: {}", &host);
    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(5))
        .build()?;
    let uri = build_url(&host);
    let resp: AlienvaultResult = client.get(&uri).send().await?.json().await?;
    debug!("alientvault response:{:?}", &resp);
    match resp.count {
        0 => Err(Error::source_error("AlienVault", host)),
        _ => Ok(resp.subdomains()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_builder() {
        let correct_uri = "https://otx.alienvault.com/api/v1/indicators/domain/\
        hackerone.com/passive_dns";
        assert_eq!(correct_uri, build_url("hackerone.com"));
    }

    // Checks to see if the run function returns subdomains
    #[tokio::test]
    async fn returns_results() {
        let host = Arc::new("hackerone.com".to_string());
        let results = run(host).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn handle_no_results() {
        let host = Arc::new("anVubmxpa2VzdGVh.com".to_string());
        let res = run(host).await;
        let e = res.unwrap_err();
        assert_eq!(
            e.to_string(),
            "AlienVault couldn't find any results for: anVubmxpa2VzdGVh.com"
        );
    }
}
