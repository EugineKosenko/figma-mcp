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
                    "ids": { "type": "array", "items": { "type": "string" }, "description": "id вузлів, напр. [\"979:3\"]" },
                    "depth": { "type": "number", "description": "Скільки рівнів показати нижче кожного вузла, за замовчуванням 2" },
                    "file": { "type": "string", "description": "Ключ файлу; без нього береться FIGMA_FILE" },
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
                    "ids": { "type": "array", "items": { "type": "string" }, "description": "id вузлів (екранів), напр. [\"979:109\", \"980:310\"]" },
                    "file": { "type": "string", "description": "Ключ файлу; без нього береться FIGMA_FILE" },
                    "refresh": { "type": "boolean", "description": "Пропустити кеш і запитати Figma наново" }
                },
                "required": ["ids"],
                "additionalProperties": false
            }
        },
        {
            "name": "comments",
            "description": "Коментарі файлу Figma гілками: номер, дата, стан, вузол і текст, під кожним — відповіді. Необов'язковий node_id лишає лише коментарі, прив'язані до цього вузла.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "node_id": { "type": "string", "description": "Лише коментарі, прив'язані до цього вузла" },
                    "file": { "type": "string", "description": "Ключ файлу; без нього береться FIGMA_FILE" },
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

pub fn file_key(arguments: &serde_json::Value) -> String {
    match arguments["file"].as_str() {
        Some(file) => file.to_string(),
        None => std::env::var("FIGMA_FILE").unwrap(),
    }
}

pub fn node_ids(arguments: &serde_json::Value) -> Vec<String> {
    arguments["ids"].as_array().unwrap().iter().map(|id| id.as_str().unwrap().to_string()).collect()
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
