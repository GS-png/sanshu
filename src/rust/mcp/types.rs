use chrono;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CacheRequest {
    #[schemars(description = "The content to display")]
    pub message: String,

    #[schemars(description = "Optional list of response templates")]
    #[serde(default)]
    pub choices: Vec<String>,

    #[schemars(description = "Enable rich text formatting, defaults to true")]
    #[serde(default = "default_chalkboard")]
    pub format: bool,

    #[schemars(description = "Project root path for context")]
    #[serde(default)]
    pub project_root_path: Option<String>,
}

fn default_chalkboard() -> bool {
    true
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StoreRequest {
    #[schemars(description = "Operation type: store (add entry), recall (get project info)")]
    pub action: String,
    #[schemars(description = "Project path (required)")]
    pub project_path: String,
    #[schemars(description = "Entry content (required for store operation)")]
    #[serde(default)]
    pub content: String,
    #[schemars(
        description = "Category: rule, preference, pattern, context"
    )]
    #[serde(default = "default_category")]
    pub category: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AcemcpRequest {
    #[schemars(description = "Absolute path to project root directory using forward slashes")]
    pub project_root_path: String,
    #[schemars(description = "Natural language search query to find relevant code context")]
    pub query: String,
}

fn default_category() -> String {
    "context".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PopupRequest {
    pub id: String,
    pub message: String,
    pub menu: Option<Vec<String>>,
    pub chalkboard: bool,
    pub project_root_path: Option<String>,
}

/// Structured response data format
#[derive(Debug, Deserialize)]
pub struct DishResponse {
    pub note: Option<String>,
    pub toppings: Vec<String>,
    #[serde(default)]
    pub ingredients: Vec<IngredientAttachment>,
    pub ticket: KitchenTicket,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IngredientAttachment {
    pub sauce: String,
    pub dish_type: String,
    pub tag: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct KitchenTicket {
    pub cooked_at: Option<String>,
    pub ticket_id: Option<String>,
    pub station: Option<String>,
}

/// Build MCP response
pub fn build_dish_response(
    note: Option<String>,
    toppings: Vec<String>,
    ingredients: Vec<IngredientAttachment>,
    ticket_id: Option<String>,
    station: &str,
) -> serde_json::Value {
    serde_json::json!({
        "note": note,
        "toppings": toppings,
        "ingredients": ingredients,
        "ticket": {
            "cooked_at": chrono::Utc::now().to_rfc3339(),
            "ticket_id": ticket_id,
            "station": station
        }
    })
}

/// Build send response
pub fn build_serve_response(
    note: Option<String>,
    toppings: Vec<String>,
    ingredients: Vec<IngredientAttachment>,
    ticket_id: Option<String>,
    station: &str,
) -> String {
    let response = build_dish_response(note, toppings, ingredients, ticket_id, station);
    response.to_string()
}

/// Build continue response
pub fn build_refill_response(ticket_id: Option<String>, station: &str) -> String {
    let continue_prompt = if let Ok(config) = crate::config::load_standalone_config() {
        config.reply_config.continue_prompt
    } else {
        "Please continue following best practices".to_string()
    };

    let response = build_dish_response(Some(continue_prompt), vec![], vec![], ticket_id, station);
    response.to_string()
}
