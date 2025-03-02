use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::request::{CompletionRequest, ContentPart, Message as ReqMessage, MessageContent};
use super::response::{Choice, Response};

/// A flattened chat message.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Extract one or more flattened messages from a given request message.
///
/// If the message’s content is an array of parts, each part becomes a separate
/// flattened message. For each content part:
///   - Text parts are extracted as is.
///   - ImageUrl parts are converted into a text message containing the image URL.
fn extract_messages(message: &ReqMessage) -> Result<Vec<Message>, String> {
    let role = message.role.clone();
    if let Some(content) = &message.content {
        match content {
            MessageContent::Text(text) => Ok(vec![Message { role, content: text.clone() }]),
            MessageContent::Parts(parts) => {
                let mut msgs = Vec::with_capacity(parts.len());
                for part in parts {
                    let part_text = match part {
                        ContentPart::Text { text } => text.clone(),
                        ContentPart::ImageUrl { image_url } => image_url.url.clone(),
                    };
                    msgs.push(Message { role: role.clone(), content: part_text });
                }
                Ok(msgs)
            }
        }
    } else {
        Err("Message is missing its content".to_string())
    }
}

/// Parse a JSON value representing a CompletionRequest into a vector of flattened messages.
///
/// - If the JSON contains a "messages" field, each message is processed.
///   If a message has an array of parts, each part becomes a separate message.
/// - If "messages" is not present but "prompt" is provided,
///   a single message is created with the role "user".
pub fn parse_chat_request(json: &Value) -> Result<Vec<Message>, String> {
    let req: CompletionRequest = serde_json::from_value(json.clone())
        .map_err(|e| format!("Failed to parse CompletionRequest: {}", e))?;

    let mut result = Vec::new();
    if let Some(messages) = req.messages {
        for message in messages {
            let mut msgs = extract_messages(&message)?;
            result.append(&mut msgs);
        }
    } else if let Some(prompt) = req.prompt {
        // If no messages array is provided, treat the prompt as a single "user" message.
        result.push(Message { role: "user".to_string(), content: prompt });
    } else {
        return Err("Neither messages nor prompt found in the request".to_string());
    }
    Ok(result)
}

/// Convert a parsed response (of one of the three variants) into a vector of flattened messages.
pub fn parse_chat_response(json: &Value) -> Result<Vec<Message>, String> {
    let response: Response = serde_json::from_value(json.clone())
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    let mut messages = Vec::new();
    for choice in response.choices {
        match choice {
            Choice::NonChat(nc) => {
                messages.push(Message { role: "assistant".to_string(), content: nc.text });
            }
            Choice::NonStreaming(ns) => {
                if let Some(content) = ns.message.content {
                    messages.push(Message { role: ns.message.role, content });
                }
            }
            Choice::Streaming(s) => {
                if let Some(content) = s.delta.content {
                    let role = s.delta.role.unwrap_or_else(|| "assistant".to_string());
                    messages.push(Message { role, content });
                }
            }
        }
    }
    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    use super::Message as FlatMessage;

    // Tests for parsing chat requests.
    #[test]
    fn test_parse_chat_request_with_string_content() {
        let request_json = json!({
            "messages": [
                { "role": "system", "content": "You are helpful." },
                { "role": "user", "content": "Hello!" }
            ]
        });
        let messages = parse_chat_request(&request_json).expect("Failed to parse request");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, "You are helpful.");
        assert_eq!(messages[1].role, "user");
        assert_eq!(messages[1].content, "Hello!");
    }

    #[test]
    fn test_parse_chat_request_with_array_content() {
        let request_json = json!({
            "messages": [
                {
                    "role": "user",
                    "content": [
                        { "type": "text", "text": "Part one." },
                        { "type": "text", "text": "Part two." },
                        { "type": "image_url", "image_url": { "url": "http://example.com/image.png" } }
                    ]
                }
            ]
        });
        let messages = parse_chat_request(&request_json).expect("Failed to parse request");
        // Expect 3 messages produced from the one input message.
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "Part one.");
        assert_eq!(messages[1].role, "user");
        assert_eq!(messages[1].content, "Part two.");
        assert_eq!(messages[2].role, "user");
        assert_eq!(messages[2].content, "http://example.com/image.png");
    }

    // Tests for parsing chat responses.
    #[test]
    fn test_parse_non_chat_choice() {
        let response_json = json!({
            "id": "resp-123",
            "created": 1610000000,
            "model": "test-model",
            "object": "chat.completion",
            "choices": [
                {
                    "finish_reason": "stop",
                    "text": "Non-chat response text."
                }
            ]
        });
        let messages = parse_chat_response(&response_json).expect("Failed to parse response");
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            FlatMessage {
                role: "assistant".to_string(),
                content: "Non-chat response text.".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_non_streaming_choice() {
        let response_json = json!({
            "id": "resp-456",
            "created": 1610000100,
            "model": "test-model",
            "object": "chat.completion",
            "choices": [
                {
                    "finish_reason": "stop",
                    "message": {
                        "role": "assistant",
                        "content": "Non-streaming chat response."
                    }
                }
            ]
        });
        let messages = parse_chat_response(&response_json).expect("Failed to parse response");
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            FlatMessage {
                role: "assistant".to_string(),
                content: "Non-streaming chat response.".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_streaming_choice() {
        let response_json = json!({
            "id": "resp-789",
            "created": 1610000200,
            "model": "test-model",
            "object": "chat.completion.chunk",
            "choices": [
                {
                    "finish_reason": null,
                    "delta": {
                        "role": "assistant",
                        "content": "Streaming chat response part."
                    }
                }
            ]
        });
        let messages = parse_chat_response(&response_json).expect("Failed to parse response");
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            FlatMessage {
                role: "assistant".to_string(),
                content: "Streaming chat response part.".to_string(),
            }
        );
    }
}
