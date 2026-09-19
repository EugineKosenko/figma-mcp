mod outline;
mod texts;
mod comments;

pub fn list() -> serde_json::Value {
    serde_json::json!([
        {
            "name": "outline",
            "description": "Структура вузлів Figma: id, тип і назва кожного вузла піддерева. Один виклик Figma на весь перелік ids.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "ids": { "type": "array", "items": { "type": "string" }, "description": "id вузлів, напр. [\"979:3\"]; форма з дефісом з адреси браузера (979-3) теж приймається" },
                    "depth": { "type": "number", "description": "Скільки рівнів показати нижче кожного вузла, за замовчуванням 2" },
                    "file": { "type": "string", "description": "Ключ або повна адреса файлу Figma; без нього береться FIGMA_FILE" },
                    "refresh": { "type": "boolean", "description": "Пропустити кеш і запитати Figma наново" }
                },
                "required": ["ids"],
                "additionalProperties": false
            }
        },
        {
            "name": "texts",
            "description": "Текстові шари вузлів Figma, згруповані за рядками екрана (зверху вниз, зліва направо): фільтри, заголовки колонок, кнопки. Один виклик Figma на весь перелік ids.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "ids": { "type": "array", "items": { "type": "string" }, "description": "id вузлів (екранів), напр. [\"979:109\", \"980:310\"]; форма з дефісом з адреси браузера (979-109) теж приймається" },
                    "file": { "type": "string", "description": "Ключ або повна адреса файлу Figma; без нього береться FIGMA_FILE" },
                    "refresh": { "type": "boolean", "description": "Пропустити кеш і запитати Figma наново" }
                },
                "required": ["ids"],
                "additionalProperties": false
            }
        },
        {
            "name": "comments",
            "description": "Коментарі файлу Figma гілками: номер, дата, стан, вузол зі зсувом (@вузол x=… y=…) і текст, під кожним — відповіді. Зсув — позиція коментаря відносно початку вузла; за ним клієнт визначає екран. Необов'язковий node_id лишає лише коментарі, прив'язані до цього вузла.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "node_id": { "type": "string", "description": "Лише коментарі, прив'язані до цього вузла; форма з дефісом (979-3) теж приймається" },
                    "file": { "type": "string", "description": "Ключ або повна адреса файлу Figma; без нього береться FIGMA_FILE" },
                    "refresh": { "type": "boolean", "description": "Пропустити кеш і запитати Figma наново" }
                },
                "additionalProperties": false
            }
        }
    ])
}

pub async fn call(http: &reqwest::Client, name: &str, arguments: &serde_json::Value) -> serde_json::Value {
    match name {
        "outline" => outline::run(http, arguments).await,
        "texts" => texts::run(http, arguments).await,
        "comments" => comments::run(http, arguments).await,
        _ => reply(Err(format!("Невідомий інструмент: {}", name)))
    }
}

fn key_from(file: &str) -> String {
    for marker in ["/design/", "/file/", "/board/"] {
        if let Some((_, rest)) = file.split_once(marker) {
            return rest.split(|c| c == '/' || c == '?' || c == '#').next().unwrap().to_string();
        }
    }

    file.to_string()
}

pub fn file_key(arguments: &serde_json::Value) -> Result<String, String> {
    match arguments["file"].as_str() {
        Some(file) => Ok(key_from(file)),
        None => std::env::var("FIGMA_FILE")
            .ok()
            .filter(|file| !file.is_empty())
            .map(|file| key_from(&file))
            .ok_or("Не вказано файл: передайте аргумент file (ключ або адресу файлу) чи задайте змінну FIGMA_FILE.".to_string()),
    }
}
pub fn node_id(id: &str) -> String {
    match id.split_once('-') {
        Some((left, right))
            if !left.is_empty()
                && !right.is_empty()
                && left.bytes().all(|b| b.is_ascii_digit())
                && right.bytes().all(|b| b.is_ascii_digit()) =>
            format!("{}:{}", left, right),
        _ => id.to_string(),
    }
}

pub fn node_ids(arguments: &serde_json::Value) -> Result<Vec<String>, String> {
    let ids: Vec<String> = match &arguments["ids"] {
        serde_json::Value::Array(items) => items.iter().filter_map(|id| id.as_str()).map(node_id).collect(),
        serde_json::Value::String(text) => text.split(',').map(|id| node_id(id.trim())).collect(),
        _ => Vec::new(),
    };

    if ids.is_empty() {
        return Err("Не вказано ids: передайте перелік id вузлів, напр. [\"979:3\"].".to_string());
    }

    Ok(ids)
}

pub fn file_and_ids(arguments: &serde_json::Value) -> Result<(String, Vec<String>), String> {
    Ok((file_key(arguments)?, node_ids(arguments)?))
}

pub fn refresh(arguments: &serde_json::Value) -> bool {
    arguments["refresh"].as_bool().unwrap_or(false)
}
pub fn reply(result: Result<String, String>) -> serde_json::Value {
    match result {
        Ok(text) => serde_json::json!({ "content": [{ "type": "text", "text": text }], "isError": false }),
        Err(text) => serde_json::json!({ "content": [{ "type": "text", "text": text }], "isError": true }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_from_address() {
        assert_eq!(key_from("https://www.figma.com/design/LismtgSCxfd7p8igMtYct3/Internal-Interfaces?node-id=979-3&t=abc"), "LismtgSCxfd7p8igMtYct3");
        assert_eq!(key_from("https://www.figma.com/file/AbC123/Name"), "AbC123");
        assert_eq!(key_from("https://www.figma.com/design/AbC123?node-id=1-2"), "AbC123");
        assert_eq!(key_from("AbC123"), "AbC123");
    }
    
    #[test]
    fn file_from_argument() {
        assert_eq!(file_key(&serde_json::json!({ "file": "https://www.figma.com/design/AbC123/N" })).unwrap(), "AbC123");
    }
    
    #[test]
    fn node_id_dash_form() {
        assert_eq!(node_id("979-3"), "979:3");
        assert_eq!(node_id("979:3"), "979:3");
        assert_eq!(node_id("I5:1;2:3"), "I5:1;2:3");
        assert_eq!(node_id("abc-1"), "abc-1");
    }
    
    #[test]
    fn node_ids_forms() {
        assert_eq!(node_ids(&serde_json::json!({ "ids": ["979-3", "980:310"] })).unwrap(), vec!["979:3", "980:310"]);
        assert_eq!(node_ids(&serde_json::json!({ "ids": "979-3, 980-310" })).unwrap(), vec!["979:3", "980:310"]);
        assert!(node_ids(&serde_json::json!({})).is_err());
        assert!(node_ids(&serde_json::json!({ "ids": [] })).is_err());
    }
}
