use crate::error::{Error, Result};
use crate::IntoSubdomain;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

#[derive(Deserialize, Hash, PartialEq, Debug, Eq)]
struct CrtshResult {
    name_value: String,
}

impl IntoSubdomain for Vec<CrtshResult> {
    fn subdomains(&self) -> HashSet<String> {
        self.iter().map(|s| s.name_value.to_owned()).collect()
    }
}

fn build_url(host: &str) -> String {
    format!("https://crt.sh/?q=%.{}&output=json", host)
}

pub async fn run(host: Arc<String>) -> Result<HashSet<String>> {
    trace!("fetching data from crt.sh for: {}", &host);
    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(4))
        .build()?;

    let uri = build_url(&host);
    let resp: Option<Vec<CrtshResult>> = client.get(&uri).send().await?.json().await?;
    debug!("crt.sh response: {:?}", &resp);

    match resp {
        Some(data) => Ok(data.subdomains()),
        None => Err(Error::source_error("Crt.sh", host)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_builder() {
        let correct_uri = "https://crt.sh/?q=%.hackerone.com&output=json";
        assert_eq!(correct_uri, build_url("hackerone.com"));
    }

    #[ignore]
    #[tokio::test]
    async fn returns_results() {
        let host = Arc::new("hackerone.com".to_owned());
        let results = run(host).await.unwrap();
        assert!(!results.is_empty());
    }

    #[ignore] // tests passing locally but failing on linux ci?
    #[tokio::test]
    async fn handle_no_results() {
        let host = Arc::new("anVubmxpa2VzdGVh.com".to_string());
        let res = run(host).await;
        let e = res.unwrap_err();
        assert_eq!(
            e.to_string(),
            "Crt.sh couldn't find any results for: anVubmxpa2VzdGVh.com"
        );
    }
}
