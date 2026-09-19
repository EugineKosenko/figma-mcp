use crate::client;
use crate::tools;

fn collect(node: &serde_json::Value, top: f64, left: f64, texts: &mut Vec<(i64, i64, String)>) {
    if node["type"] == "TEXT" {
        let bounds = &node["absoluteBoundingBox"];
        texts.push((
            (bounds["y"].as_f64().unwrap() - top) as i64,
            (bounds["x"].as_f64().unwrap() - left) as i64,
            node["characters"].as_str().unwrap().replace('\n', " "),
        ));
    }

    if let Some(children) = node["children"].as_array() {
        for child in children {
            collect(child, top, left, texts);
        }
    }
}
fn rows(mut texts: Vec<(i64, i64, String)>) -> Vec<String> {
    texts.sort();

    let mut rows: Vec<(i64, Vec<(i64, String)>)> = Vec::new();
    for (y, x, text) in texts {
        let same = rows.last().is_some_and(|(start, _)| y - *start <= 25);

        if same {
            rows.last_mut().unwrap().1.push((x, text));
        } else {
            rows.push((y, vec![(x, text)]));
        }
    }

    rows.into_iter()
        .map(|(y, mut items)| {
            items.sort();
            format!("y={:<4} {}", y, items.into_iter().map(|(_, text)| text).collect::<Vec<_>>().join(" | "))
        })
        .collect()
}

pub async fn run(http: &reqwest::Client, arguments: &serde_json::Value) -> serde_json::Value {
    let (file, ids) = match tools::file_and_ids(arguments) {
        Ok(pair) => pair,
        Err(message) => return tools::reply(Err(message)),
    };
    
    tools::reply(
        client::file_nodes(http, &file, &ids, tools::refresh(arguments)).await.map(|body| {
            let mut found = Vec::new();
            client::find(&body["document"], &ids, &mut found);
    
            let mut blocks = Vec::new();
            for node in found {
                let bounds = &node["absoluteBoundingBox"];
                let mut texts = Vec::new();
                collect(node, bounds["y"].as_f64().unwrap(), bounds["x"].as_f64().unwrap(), &mut texts);
    
                blocks.push(format!(
                    "== {} ({}): {} текстів\n{}",
                    node["name"].as_str().unwrap(),
                    node["id"].as_str().unwrap(),
                    texts.len(),
                    rows(texts).join("\n"),
                ));
            }
    
            if blocks.is_empty() { "Вузли не знайдено.".to_string() } else { blocks.join("\n\n") }
        })
    )
}
