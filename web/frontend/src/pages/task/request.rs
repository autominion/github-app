use serde::{Deserialize, Serialize};

/// Minimal version of an OpenAI-compatible completion request,
/// supporting either a list of messages or a single prompt.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CompletionRequest {
    /// Either "messages" or "prompt" is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<Message>>,

    /// Optional prompt in case messages are not provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
}

/// Represents a single message. For roles "user", "assistant", or "system"
/// the content can either be a plain string or an array of parts.
/// For a "tool" message, content should be a plain string.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Message {
    /// "user" | "assistant" | "system" | "tool"
    pub role: String,

    /// Content depends on the role.
    /// - For user/assistant/system, it can be either a string or an array of parts.
    /// - For tool, it is expected to be a plain string.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<MessageContent>,

    /// Optional field used with tool messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,

    /// Optional name field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// The content of a message can either be a simple text string
/// or a list of structured content parts.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

/// A content part is one of:
/// - Text: `{ type: "text", text: "..." }`
/// - Image URL: `{ type: "image_url", image_url: { url: "...", detail?: "..." } }`
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}

/// Contains an image URL (or base64-encoded data) plus an optional detail.
#[derive(Debug, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
}
