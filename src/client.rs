use std::env;
use std::path::PathBuf;

const API: &str = "https://api.figma.com/v1";
fn fingerprint(text: &str) -> u64 {
    text.bytes().fold(0xcbf29ce484222325, |hash, byte| (hash ^ byte as u64).wrapping_mul(0x100000001b3))
}

fn cache_path(url: &str) -> PathBuf {
    let dir = match env::var("FIGMA_CACHE") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => env::temp_dir().join("figma-mcp"),
    };

    dir.join(format!("{:016x}.json", fingerprint(url)))
}
fn header(response: &reqwest::Response, name: &str) -> String {
    match response.headers().get(name) {
        Some(value) => value.to_str().unwrap().to_string(),
        None => "?".to_string(),
    }
}
async fn get(http: &reqwest::Client, path: &str, refresh: bool) -> Result<serde_json::Value, String> {
    let url = format!("{}{}", API, path);
    let cache = cache_path(&url);

    if !refresh {
        if let Ok(text) = std::fs::read_to_string(&cache) {
            return Ok(serde_json::from_str(&text).unwrap());
        }
    }

    let response = http
        .get(&url)
        .header("X-Figma-Token", env::var("FIGMA_TOKEN").unwrap())
        .send()
        .await.unwrap();
    let status = response.status();

    if status.as_u16() == 429 {
        let seconds = header(&response, "retry-after").parse::<u64>().unwrap_or(0);
        return Err(format!(
            "Ліміт Figma вичерпано (429) для {}: повторіть приблизно через {} год. План: {}, тип ліміту: {}.",
            path,
            seconds / 3600,
            header(&response, "x-figma-plan-tier"),
            header(&response, "x-figma-rate-limit-type"),
        ));
    }

    let body = response.text().await.unwrap();

    if !status.is_success() {
        return Err(format!("Figma відповів {} для {}: {}", status, path, body));
    }

    std::fs::create_dir_all(cache.parent().unwrap()).unwrap();
    std::fs::write(&cache, &body).unwrap();

    Ok(serde_json::from_str(&body).unwrap())
}
pub async fn file_nodes(http: &reqwest::Client, file: &str, ids: &[String], refresh: bool) -> Result<serde_json::Value, String> {
    get(http, &format!("/files/{}?ids={}", file, ids.join(",")), refresh).await
}
pub fn find<'a>(node: &'a serde_json::Value, ids: &[String], found: &mut Vec<&'a serde_json::Value>) {
    if ids.iter().any(|id| node["id"] == *id) {
        found.push(node);
        return;
    }

    if let Some(children) = node["children"].as_array() {
        for child in children {
            find(child, ids, found);
        }
    }
}
pub async fn comments(http: &reqwest::Client, file: &str, refresh: bool) -> Result<serde_json::Value, String> {
    get(http, &format!("/files/{}/comments", file), refresh).await
}
