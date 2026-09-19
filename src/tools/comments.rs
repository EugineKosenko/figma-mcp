use crate::client;
use crate::tools;

fn line(comment: &serde_json::Value) -> String {
    comment["message"].as_str().unwrap().split_whitespace().collect::<Vec<_>>().join(" ")
}

fn number(comment: &serde_json::Value) -> String {
    comment["order_id"].to_string().trim_matches('"').to_string()
}
fn anchor(comment: &serde_json::Value) -> String {
    let node = comment.pointer("/client_meta/node_id").and_then(|id| id.as_str()).unwrap_or("-");
    let x = comment.pointer("/client_meta/node_offset/x").and_then(|v| v.as_f64());
    let y = comment.pointer("/client_meta/node_offset/y").and_then(|v| v.as_f64());

    match (x, y) {
        (Some(x), Some(y)) => format!("@{} x={} y={}", node, x, y),
        _ => format!("@{}", node),
    }
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
                    "#{} {} {} {}: {}",
                    number(root),
                    &root["created_at"].as_str().unwrap()[..10],
                    if root["resolved_at"].is_null() { "відкрито" } else { "закрито" },
                    anchor(root),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_with_offset() {
        let comment = serde_json::json!({ "client_meta": { "node_id": "979:3", "node_offset": { "x": 500, "y": 5471 } } });
        assert_eq!(anchor(&comment), "@979:3 x=500 y=5471");
    }
    
    #[test]
    fn anchor_fractional_and_negative_offset() {
        let comment = serde_json::json!({ "client_meta": { "node_id": "0:1", "node_offset": { "x": 12.5, "y": -3 } } });
        assert_eq!(anchor(&comment), "@0:1 x=12.5 y=-3");
    }
    
    #[test]
    fn anchor_without_offset_or_node() {
        assert_eq!(anchor(&serde_json::json!({ "client_meta": { "node_id": "979:3" } })), "@979:3");
        assert_eq!(anchor(&serde_json::json!({})), "@-");
    }
}
