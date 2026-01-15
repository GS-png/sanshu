use anyhow::Result;
use rmcp::model::{ErrorData as McpError, Content};

use crate::mcp::types::DishResponse;

const MAX_SAUCE_BASE64_LEN_FOR_OUTPUT: usize = 800_000;

fn estimate_base64_size_str(base64_len: usize) -> String {
    let estimated_size = (base64_len * 3) / 4;
    if estimated_size < 1024 {
        format!("{} B", estimated_size)
    } else if estimated_size < 1024 * 1024 {
        format!("{:.1} KB", estimated_size as f64 / 1024.0)
    } else {
        format!("{:.1} MB", estimated_size as f64 / (1024.0 * 1024.0))
    }
}

fn is_supported_dish_type(dish_type: &str) -> bool {
    matches!(dish_type, "image/png" | "image/jpeg" | "image/webp")
}

/// Parse MCP response content
pub fn parse_mcp_response(response: &str) -> Result<Vec<Content>, McpError> {
    let trimmed = response.trim();
    if trimmed == "CANCELLED" || trimmed == "\"CANCELLED\"" {
        return Ok(vec![Content::text("Operation cancelled by user".to_string())]);
    }

    // Try structured format first
    if let Ok(structured_response) = serde_json::from_str::<DishResponse>(response) {
        return parse_structured_response(structured_response);
    }

    Ok(vec![Content::text(response.to_string())])
}

/// Parse structured response format
fn parse_structured_response(response: DishResponse) -> Result<Vec<Content>, McpError> {
    let mut result = Vec::new();
    let mut text_parts = Vec::new();

    if !response.toppings.is_empty() {
        text_parts.push(format!("Selected: {}", response.toppings.join(", ")));
    }
    if let Some(note) = response.note {
        if !note.trim().is_empty() {
            text_parts.push(note.trim().to_string());
        }
    }

    let mut ingredient_info_parts = Vec::new();
    for (index, ingredient) in response.ingredients.iter().enumerate() {
        let base64_len = ingredient.sauce.len();

        let size_str = estimate_base64_size_str(base64_len);
        let is_supported = is_supported_dish_type(ingredient.dish_type.as_str());
        let is_too_large = base64_len > MAX_SAUCE_BASE64_LEN_FOR_OUTPUT;

        if is_supported && !is_too_large {
            result.push(Content::image(ingredient.sauce.clone(), ingredient.dish_type.clone()));
        }

        let note = if !is_supported {
            "(已省略：食材类型不受支持，建议用 PNG/JPG/WebP)"
        } else if is_too_large {
            "(已省略：食材过大，避免触发参数超限)"
        } else {
            ""
        };

        let filename_info = ingredient
            .tag
            .as_ref()
            .map(|f| format!("\nFilename: {}", f))
            .unwrap_or_default();

        let ingredient_info = format!(
            "=== Ingredient {} ==={}\nType: {}\nSize: {}\nBase64 length: {} chars {}",
            index + 1,
            filename_info,
            ingredient.dish_type,
            size_str,
            base64_len,
            note
        );
        ingredient_info_parts.push(ingredient_info);
    }

    let mut all_text_parts = text_parts;
    all_text_parts.extend(ingredient_info_parts);

    if !response.ingredients.is_empty() {
        all_text_parts.push(format!(
            "Note: User provided {} ingredient(s).",
            response.ingredients.len()
        ));
    }

    if !all_text_parts.is_empty() {
        let combined_text = all_text_parts.join("\n\n");
        result.push(Content::text(combined_text));
    }

    if result.is_empty() {
        result.push(Content::text("No content provided".to_string()));
    }

    Ok(result)
}
