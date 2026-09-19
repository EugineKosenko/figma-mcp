use crate::client;
use crate::tools;

fn line(comment: &serde_json::Value) -> String {
    comment["message"].as_str().unwrap().split_whitespace().collect::<Vec<_>>().join(" ")
}

fn number(comment: &serde_json::Value) -> String {
    comment["order_id"].to_string().trim_matches('"').to_string()
}

pub async fn run(http: &reqwest::Client, arguments: &serde_json::Value) -> serde_json::Value {
    let file = match tools::file_key(arguments) {
        Ok(file) => file,
        Err(message) => return tools::reply(Err(message)),
    };
    let node_filter = arguments["node_id"].as_str().map(tools::node_id);
    
    tools::reply(
        client::comments(http, &file, tools::refresh(arguments)).await.map(|body| {
            let all = body["comments"].as_array().unwrap();
    
            let mut roots: Vec<&serde_json::Value> = all
                .iter()
                .filter(|c| c["parent_id"].as_str().is_none_or(|parent| parent.is_empty()))
                .filter(|c| node_filter.as_ref().is_none_or(|node| c.pointer("/client_meta/node_id").and_then(|id| id.as_str()) == Some(node.as_str())))
                .collect();
            roots.sort_by_key(|c| c["created_at"].as_str().unwrap().to_string());
    
            let mut lines = Vec::new();
            for root in roots {
                lines.push(format!(
                    "#{} {} {} @{}: {}",
                    number(root),
                    &root["created_at"].as_str().unwrap()[..10],
                    if root["resolved_at"].is_null() { "відкрито" } else { "закрито" },
                    root.pointer("/client_meta/node_id").and_then(|id| id.as_str()).unwrap_or("-"),
                    line(root),
                ));
    
                let mut replies: Vec<&serde_json::Value> = all.iter().filter(|c| c["parent_id"] == root["id"]).collect();
                replies.sort_by_key(|c| c["created_at"].as_str().unwrap().to_string());
                for reply in replies {
                    lines.push(format!("  ↳ {}", line(reply)));
                }
            }
    
            if lines.is_empty() { "Коментарів не знайдено.".to_string() } else { lines.join("\n") }
        })
    )
}
