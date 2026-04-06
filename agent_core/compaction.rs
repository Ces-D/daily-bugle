/// Configuration for automatic context compaction.
#[derive(Debug, Clone)]
pub struct CompactionConfig {
    /// Estimated token budget. When the conversation exceeds this, compaction triggers.
    /// Should be set below the model's context window to leave room for the next response.
    /// Default: 80_000 (suitable for 128k context windows).
    pub token_budget: usize,

    /// Number of recent messages to preserve verbatim (not summarized).
    /// These are the most recent messages that the model needs full detail on.
    /// Default: 6 (approximately 3 user+assistant turn pairs).
    pub preserve_recent: usize,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            token_budget: 80_000,
            preserve_recent: 6,
        }
    }
}

/// Rough token estimate for a single ChatMessage.
///
/// Uses the heuristic of ~4 characters per token, which is a reasonable
/// average across English text for most tokenizers (GPT, Claude, etc.).
/// This is intentionally imprecise — compaction is a soft threshold,
/// not a hard limit.
fn estimate_message_tokens(message: &genai::chat::ChatMessage) -> usize {
    let text = serde_json::to_string(&message.content).unwrap_or_default();
    text.len() / 4
}

/// Estimate the total token count across all messages.
pub fn estimate_total_tokens(messages: &[genai::chat::ChatMessage]) -> usize {
    messages.iter().map(|m| estimate_message_tokens(m)).sum()
}

/// Returns true if the estimated token count exceeds the budget.
pub fn should_compact(messages: &[genai::chat::ChatMessage], config: &CompactionConfig) -> bool {
    if messages.len() <= config.preserve_recent {
        return false;
    }
    estimate_total_tokens(messages) > config.token_budget
}

/// Compact the conversation by summarizing old messages.
///
/// Splits the message list into two halves:
/// - Old messages (everything before the last `preserve_recent`): summarized by the LLM.
/// - Recent messages (last `preserve_recent`): kept verbatim.
///
/// The summary replaces all old messages with a single system message containing
/// the condensed conversation history.
///
/// Returns the compacted message list: [summary_system_message, ...recent_messages].
pub async fn compact(
    client: &genai::Client,
    model: &str,
    messages: &[genai::chat::ChatMessage],
    config: &CompactionConfig,
) -> anyhow::Result<Vec<genai::chat::ChatMessage>> {
    use genai::chat::{ChatMessage, ChatRequest};

    if messages.len() <= config.preserve_recent {
        return Ok(messages.to_vec());
    }

    let split_point = messages.len() - config.preserve_recent;
    let old_messages = &messages[..split_point];
    let recent_messages = &messages[split_point..];

    let summary_system = ChatMessage::system(
        "You are a conversation summarizer. Summarize the following conversation history \
         into a concise but complete summary. Preserve:\n\
         - All key facts, decisions, and conclusions reached\n\
         - Tool call results and their outcomes\n\
         - Any commitments, plans, or action items discussed\n\
         - User preferences or corrections mentioned\n\n\
         Be concise but do not omit important details. Output only the summary, \
         no preamble.",
    );

    let mut summary_messages = vec![summary_system];
    summary_messages.extend_from_slice(old_messages);
    summary_messages.push(ChatMessage::user("Summarize the conversation above."));

    let summary_request = ChatRequest::from_messages(summary_messages);
    let summary_response = client
        .exec_chat(model, summary_request, None)
        .await
        .map_err(|e| anyhow::anyhow!("Compaction LLM call failed: {e}"))?;

    let summary_text = summary_response
        .first_text()
        .unwrap_or("[Compaction failed: no summary text returned]");

    let mut compacted = Vec::with_capacity(1 + recent_messages.len());
    compacted.push(ChatMessage::system(format!(
        "[Conversation summary — compacted from {} messages]\n\n{}",
        old_messages.len(),
        summary_text
    )));
    compacted.extend_from_slice(recent_messages);

    log::info!(
        "Compacted conversation: {} messages -> {} messages (summarized {} old messages)",
        messages.len(),
        compacted.len(),
        old_messages.len(),
    );

    Ok(compacted)
}
