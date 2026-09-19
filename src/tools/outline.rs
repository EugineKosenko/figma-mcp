use crate::client;
use crate::tools;

fn walk(node: &serde_json::Value, level: u64, depth: u64, lines: &mut Vec<String>) {
    lines.push(format!(
        "{}{} {} {}",
        "  ".repeat(level as usize),
        node["id"].as_str().unwrap(),
        node["type"].as_str().unwrap(),
        node["name"].as_str().unwrap(),
    ));

    if level < depth {
        if let Some(children) = node["children"].as_array() {
            for child in children {
                walk(child, level + 1, depth, lines);
            }
        }
    }
}

pub async fn run(http: &reqwest::Client, arguments: &serde_json::Value) -> serde_json::Value {
    let (file, ids) = match tools::file_and_ids(arguments) {
        Ok(pair) => pair,
        Err(message) => return tools::reply(Err(message)),
    };
    let depth = arguments["depth"].as_u64().unwrap_or(2);
    
    tools::reply(
        client::file_nodes(http, &file, &ids, tools::refresh(arguments)).await.map(|body| {
            let mut found = Vec::new();
            client::find(&body["document"], &ids, &mut found);
    
            let mut lines = Vec::new();
            for node in found {
                walk(node, 0, depth, &mut lines);
            }
    
            if lines.is_empty() { "Вузли не знайдено.".to_string() } else { lines.join("\n") }
        })
    )
}
