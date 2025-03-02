use serde::Deserialize;

/// Represents the overall response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Response {
    pub choices: Vec<Choice>,
}

/// The response choices may come in several variants.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Choice {
    /// For non-chat responses.
    NonChat(NonChatChoice),
    /// For non-streaming chat responses.
    NonStreaming(NonStreamingChoice),
    /// For streaming responses.
    Streaming(StreamingChoice),
}

/// A non-chat response choice. (Typically used when a prompt is given instead of messages.)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NonChatChoice {
    pub text: String,
}

/// A non-streaming chat response choice.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NonStreamingChoice {
    pub message: ChatMessage,
}

/// A streaming response choice.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StreamingChoice {
    pub delta: Delta,
}

/// The chat message within a non-streaming choice.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChatMessage {
    pub content: Option<String>,
    pub role: String,
}

/// The delta update in a streaming choice.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Delta {
    pub content: Option<String>,
    pub role: Option<String>,
}
