use reqwest;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug)]
pub enum LlmError {
    NetworkError(reqwest::Error),
    ApiError(String),
    ParseError(String),
    InvalidResponse,
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LlmError::NetworkError(e) => write!(f, "Network error: {}", e),
            LlmError::ApiError(msg) => write!(f, "API error: {}", msg),
            LlmError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            LlmError::InvalidResponse => write!(f, "Invalid response from API"),
        }
    }
}

impl std::error::Error for LlmError {}

impl From<reqwest::Error> for LlmError {
    fn from(err: reqwest::Error) -> Self {
        LlmError::NetworkError(err)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug)]
pub struct RecipeInfo {
    pub ingredients: Vec<String>,
    pub prep_time: Option<String>,
    pub cook_time: Option<String>,
    pub total_time: Option<String>,
    pub servings: Option<String>,
}

pub async fn extract_recipe_info(
    page_content: &str,
    api_key: &str,
) -> Result<RecipeInfo, LlmError> {
    let prompt = format!(
        "Please analyze this recipe content and extract the following information in a structured format:
        
        1. Ingredients list (clean, human-readable format)
        2. Preparation time
        3. Cooking time  
        4. Total time
        5. Number of servings
        
        Return the information in this exact JSON format:
        {{
            \"ingredients\": [\"ingredient 1\", \"ingredient 2\", ...],
            \"prep_time\": \"time or null\",
            \"cook_time\": \"time or null\", 
            \"total_time\": \"time or null\",
            \"servings\": \"servings or null\"
        }}
        
        Recipe content:
        {}",
        page_content
    );

    let request = ChatRequest {
        model: "openai/gpt-4o".to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant that extracts recipe information from web content. Always return valid JSON without markdown code blocks or formatting.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ],
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        println!("LLM API Error Response: {}", error_text);
        return Err(LlmError::ApiError(format!(
            "HTTP {}: {}",
            status, error_text
        )));
    }

    println!("LLM API Response Status: {}", response.status());
    // raw dump the returned json

    let chat_response: ChatResponse = response
        .json()
        .await
        .map_err(|e| LlmError::ParseError(format!("Failed to parse JSON response: {}", e)))?;

    if chat_response.choices.is_empty() {
        return Err(LlmError::InvalidResponse);
    }

    let content = &chat_response.choices[0].message.content;

    // Clean up the content - remove markdown code blocks if present
    let cleaned_content = content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    // Parse the JSON response from the LLM
    let recipe_data: serde_json::Value = serde_json::from_str(cleaned_content)
        .map_err(|e| LlmError::ParseError(format!("Failed to parse recipe JSON: {}", e)))?;

    let ingredients = recipe_data["ingredients"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .collect();

    let get_string = |key: &str| {
        recipe_data[key]
            .as_str()
            .filter(|s| !s.is_empty() && s.to_lowercase() != "null")
            .map(|s| s.trim().to_string())
    };

    Ok(RecipeInfo {
        ingredients,
        prep_time: get_string("prep_time"),
        cook_time: get_string("cook_time"),
        total_time: get_string("total_time"),
        servings: get_string("servings"),
    })
}
