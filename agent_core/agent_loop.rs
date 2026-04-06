#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// Emitted once when the agent loop starts.
    AgentStart,

    /// Emitted once when the agent loop ends (naturally or due to error/cancellation).
    AgentEnd,

    /// Emitted at the start of each turn (one turn = one LLM call + its tool executions).
    TurnStart { turn_index: usize },

    /// Emitted at the end of each turn.
    TurnEnd { turn_index: usize },

    /// Emitted when the assistant response stream begins.
    MessageStart,

    /// Emitted for each text chunk received during streaming.
    MessageDelta { text: String },

    /// Emitted when the assistant response stream completes.
    /// Contains the full assistant ChatMessage (with tool calls if any).
    MessageEnd { message: genai::chat::ChatMessage },

    /// Emitted before a tool starts executing.
    ToolExecutionStart {
        call_id: String,
        tool_name: String,
        arguments: serde_json::Value,
    },

    /// Emitted for progress updates during tool execution (optional, tool-driven).
    ToolExecutionUpdate {
        call_id: String,
        data: serde_json::Value,
    },

    /// Emitted when a tool finishes executing.
    ToolExecutionEnd {
        call_id: String,
        tool_name: String,
        result: String,
        is_error: bool,
    },

    /// Emitted when automatic compaction starts.
    CompactionStart {
        estimated_tokens: usize,
        message_count: usize,
    },

    /// Emitted when automatic compaction completes.
    CompactionEnd {
        original_count: usize,
        compacted_count: usize,
    },

    /// Emitted when the agent is aborted (either during streaming or tool execution).
    Aborted {
        /// Where the abort occurred: "streaming" or "tool_execution".
        phase: String,
        /// If aborted during tool execution, the tool call ID.
        tool_call_id: Option<String>,
    },
}

#[async_trait::async_trait]
pub trait AgentTool: Send + Sync {
    /// The tool name. Must match the name in `definition()`.
    fn name(&self) -> &str;

    /// Returns the genai `Tool` definition (name, description, JSON schema).
    /// This is sent to the LLM so it knows what tools are available.
    fn definition(&self) -> genai::chat::Tool;

    /// Whether this tool only reads state and has no side effects.
    ///
    /// Read-only tools are safe to execute in parallel with other read-only tools.
    /// Mutating tools are always executed sequentially, with cancellation checks
    /// between each one.
    ///
    /// Default: `false` (assumes the tool may have side effects).
    fn is_read_only(&self) -> bool {
        false
    }

    /// Execute the tool with the given arguments.
    ///
    /// - `call_id`: the unique ID from the LLM's ToolCall, used to correlate results.
    /// - `arguments`: the JSON arguments from the LLM's ToolCall (`ToolCall.fn_arguments`).
    /// - `event_tx`: channel to emit `ToolExecutionUpdate` events for progress reporting.
    /// - `cancel`: cancellation token — long-running tools should check `cancel.is_cancelled()`.
    ///
    /// Returns the result as a String. This becomes the `ToolResponse.content` sent back to the LLM.
    /// Errors are caught by the loop and sent to the LLM as error text (the loop does not abort).
    async fn execute(
        &self,
        call_id: &str,
        arguments: serde_json::Value,
        event_tx: &tokio::sync::mpsc::UnboundedSender<AgentEvent>,
        cancel: &tokio_util::sync::CancellationToken,
    ) -> anyhow::Result<String>;
}

pub struct AgentLoopConfig {
    /// The model identifier (e.g., "gpt-4o", "claude-sonnet-4-20250514").
    pub model: String,

    /// System prompt sent with every LLM request.
    pub system_prompt: String,

    /// Available tools. Each tool's `definition()` is sent to the LLM.
    pub tools: Vec<Box<dyn AgentTool>>,

    /// Maximum number of turns (LLM calls) before the loop stops.
    /// Prevents infinite loops. Default: 10.
    pub max_turns: usize,

    /// Chat options passed to genai (temperature, max_tokens, capture flags, etc.).
    /// Important: `capture_content` and `capture_tool_calls` should be true.
    pub chat_options: genai::chat::ChatOptions,

    /// Compaction configuration. Set to `None` to disable automatic compaction.
    /// When enabled, the agent will summarize old messages when the estimated
    /// token count exceeds the configured budget.
    pub compaction: Option<crate::compaction::CompactionConfig>,
}

impl Default for AgentLoopConfig {
    fn default() -> Self {
        Self {
            model: String::from("gpt-4o"),
            system_prompt: String::new(),
            tools: Vec::new(),
            max_turns: 10,
            chat_options: genai::chat::ChatOptions::default()
                .with_capture_content(true)
                .with_capture_usage(true)
                .with_capture_tool_calls(true),
            compaction: Some(crate::compaction::CompactionConfig::default()),
        }
    }
}

struct AgentLoopContext {
    /// Full conversation history. Grows as the loop progresses.
    messages: Vec<genai::chat::ChatMessage>,

    /// Current turn number (0-indexed). Incremented each time the LLM is called.
    turn_index: usize,
}

fn check_cancelled(cancel: &tokio_util::sync::CancellationToken) -> anyhow::Result<()> {
    anyhow::ensure!(!cancel.is_cancelled(), "Agent loop cancelled");
    Ok(())
}

pub async fn agent_loop(
    client: &genai::Client,
    config: &AgentLoopConfig,
    initial_messages: Vec<genai::chat::ChatMessage>,
    event_tx: tokio::sync::mpsc::UnboundedSender<AgentEvent>,
    cancel: tokio_util::sync::CancellationToken,
    on_persist: Option<crate::session::PersistFn>,
) -> anyhow::Result<Vec<genai::chat::ChatMessage>> {
    let mut ctx = AgentLoopContext {
        messages: initial_messages,
        turn_index: 0,
    };

    if let Some(persist) = on_persist.as_ref() {
        persist(&ctx.messages)?;
    }

    event_tx.send(AgentEvent::AgentStart)?;
    run_loop(
        client,
        config,
        &mut ctx,
        &event_tx,
        &cancel,
        on_persist.as_ref(),
    )
    .await?;
    event_tx.send(AgentEvent::AgentEnd)?;

    Ok(ctx.messages)
}

pub async fn agent_loop_continue(
    client: &genai::Client,
    config: &AgentLoopConfig,
    mut history: Vec<genai::chat::ChatMessage>,
    follow_up: Vec<genai::chat::ChatMessage>,
    event_tx: tokio::sync::mpsc::UnboundedSender<AgentEvent>,
    cancel: tokio_util::sync::CancellationToken,
    on_persist: Option<crate::session::PersistFn>,
) -> anyhow::Result<Vec<genai::chat::ChatMessage>> {
    anyhow::ensure!(
        !history.is_empty(),
        "Cannot continue: no messages in history"
    );
    anyhow::ensure!(
        history.last().map(|m| &m.role) != Some(&genai::chat::ChatRole::Assistant),
        "Cannot continue from assistant message"
    );

    history.extend(follow_up);

    let mut ctx = AgentLoopContext {
        messages: history,
        turn_index: 0,
    };

    event_tx.send(AgentEvent::AgentStart)?;
    run_loop(
        client,
        config,
        &mut ctx,
        &event_tx,
        &cancel,
        on_persist.as_ref(),
    )
    .await?;
    event_tx.send(AgentEvent::AgentEnd)?;

    Ok(ctx.messages)
}

async fn run_loop(
    client: &genai::Client,
    config: &AgentLoopConfig,
    ctx: &mut AgentLoopContext,
    event_tx: &tokio::sync::mpsc::UnboundedSender<AgentEvent>,
    cancel: &tokio_util::sync::CancellationToken,
    on_persist: Option<&crate::session::PersistFn>,
) -> anyhow::Result<()> {
    loop {
        // Guard: max turns
        if ctx.turn_index >= config.max_turns {
            log::warn!("Agent loop reached max turns ({})", config.max_turns);
            break;
        }

        check_cancelled(cancel)?;

        // --- Compaction check ---
        if let Some(ref compaction_config) = config.compaction {
            if crate::compaction::should_compact(&ctx.messages, compaction_config) {
                let estimated_tokens = crate::compaction::estimate_total_tokens(&ctx.messages);
                event_tx.send(AgentEvent::CompactionStart {
                    estimated_tokens,
                    message_count: ctx.messages.len(),
                })?;

                let original_count = ctx.messages.len();
                ctx.messages = crate::compaction::compact(
                    client,
                    &config.model,
                    &ctx.messages,
                    compaction_config,
                )
                .await?;

                event_tx.send(AgentEvent::CompactionEnd {
                    original_count,
                    compacted_count: ctx.messages.len(),
                })?;

                if let Some(persist) = on_persist {
                    persist(&ctx.messages)?;
                }
            }
        }

        // --- Turn Start ---
        event_tx.send(AgentEvent::TurnStart {
            turn_index: ctx.turn_index,
        })?;

        // --- Stream assistant response ---
        let assistant_message =
            stream_assistant_response(client, config, &ctx.messages, event_tx, cancel).await?;

        ctx.messages.push(assistant_message.clone());

        if let Some(persist) = on_persist {
            persist(&ctx.messages)?;
        }

        // --- Extract tool calls ---
        let tool_calls = assistant_message.content.tool_calls();

        if tool_calls.is_empty() {
            // No tool calls — agent is done
            event_tx.send(AgentEvent::TurnEnd {
                turn_index: ctx.turn_index,
            })?;
            break;
        }

        // --- Execute tool calls ---
        let tool_responses =
            execute_tool_calls(&tool_calls, &config.tools, event_tx, cancel).await?;

        // --- Append tool responses as messages ---
        for response in tool_responses {
            ctx.messages.push(genai::chat::ChatMessage::from(response));
        }

        if let Some(persist) = on_persist {
            persist(&ctx.messages)?;
        }

        // --- Turn End ---
        event_tx.send(AgentEvent::TurnEnd {
            turn_index: ctx.turn_index,
        })?;
        ctx.turn_index += 1;
    }

    Ok(())
}

fn build_chat_request(
    config: &AgentLoopConfig,
    messages: &[genai::chat::ChatMessage],
) -> genai::chat::ChatRequest {
    let mut request = genai::chat::ChatRequest::from_messages(messages.to_vec());

    if !config.system_prompt.is_empty() {
        request = request.with_system(&config.system_prompt);
    }

    // Add tool definitions from all registered tools
    let tool_defs: Vec<genai::chat::Tool> = config.tools.iter().map(|t| t.definition()).collect();
    if !tool_defs.is_empty() {
        request = request.with_tools(tool_defs);
    }

    request
}

async fn stream_assistant_response(
    client: &genai::Client,
    config: &AgentLoopConfig,
    messages: &[genai::chat::ChatMessage],
    event_tx: &tokio::sync::mpsc::UnboundedSender<AgentEvent>,
    cancel: &tokio_util::sync::CancellationToken,
) -> anyhow::Result<genai::chat::ChatMessage> {
    use futures::StreamExt;

    let chat_req = build_chat_request(config, messages);
    let response = client
        .exec_chat_stream(&config.model, chat_req, Some(&config.chat_options))
        .await?;

    let mut stream = response.stream;
    let mut stream_end: Option<genai::chat::StreamEnd> = None;

    event_tx.send(AgentEvent::MessageStart)?;

    loop {
        let event = tokio::select! {
            event = stream.next() => {
                match event {
                    Some(event) => event,
                    None => break, // Stream ended
                }
            }
            () = cancel.cancelled() => {
                event_tx.send(AgentEvent::Aborted {
                    phase: "streaming".to_string(),
                    tool_call_id: None,
                })?;
                event_tx.send(AgentEvent::MessageEnd {
                    message: genai::chat::ChatMessage::assistant("Agent aborted during streaming"),
                })?;
                anyhow::bail!("Agent loop cancelled during streaming");
            }
        };

        match event? {
            genai::chat::ChatStreamEvent::Start => {}
            genai::chat::ChatStreamEvent::Chunk(chunk) => {
                event_tx.send(AgentEvent::MessageDelta {
                    text: chunk.content.clone(),
                })?;
            }
            genai::chat::ChatStreamEvent::ReasoningChunk(_) => {}
            genai::chat::ChatStreamEvent::ThoughtSignatureChunk(_) => {}
            genai::chat::ChatStreamEvent::ToolCallChunk(_) => {}
            genai::chat::ChatStreamEvent::End(end) => {
                stream_end = Some(end);
            }
        }
    }

    let end = stream_end.ok_or_else(|| anyhow::anyhow!("Stream ended without End event"))?;

    let fallback_content = end
        .captured_content
        .as_ref()
        .cloned()
        .unwrap_or_else(|| genai::chat::MessageContent::from_text(""));
    let assistant_msg = end
        .into_assistant_message_for_tool_use()
        .unwrap_or_else(|| genai::chat::ChatMessage::assistant(fallback_content));

    event_tx.send(AgentEvent::MessageEnd {
        message: assistant_msg.clone(),
    })?;

    Ok(assistant_msg)
}

async fn execute_tool_calls(
    tool_calls: &[&genai::chat::ToolCall],
    tools: &[Box<dyn AgentTool>],
    event_tx: &tokio::sync::mpsc::UnboundedSender<AgentEvent>,
    cancel: &tokio_util::sync::CancellationToken,
) -> anyhow::Result<Vec<genai::chat::ToolResponse>> {
    let mut results: Vec<Option<genai::chat::ToolResponse>> = vec![None; tool_calls.len()];
    let mut read_only_batch: Vec<(usize, &genai::chat::ToolCall)> = Vec::new();

    for (idx, tc) in tool_calls.iter().enumerate() {
        let is_read_only = tools
            .iter()
            .find(|t| t.name() == tc.fn_name)
            .is_some_and(|t| t.is_read_only());

        if is_read_only {
            read_only_batch.push((idx, tc));
        } else {
            // Safety barrier: flush read-only batch before running mutating tool
            if !read_only_batch.is_empty() {
                flush_read_only_batch(&read_only_batch, tools, event_tx, cancel, &mut results)
                    .await?;
                read_only_batch.clear();
            }

            check_cancelled(cancel)?;

            results[idx] = Some(execute_single_tool(tc, tools, event_tx, cancel).await?);
        }
    }

    // Flush any remaining read-only tools
    if !read_only_batch.is_empty() {
        flush_read_only_batch(&read_only_batch, tools, event_tx, cancel, &mut results).await?;
    }

    Ok(results
        .into_iter()
        .enumerate()
        .map(|(i, r)| r.unwrap_or_else(|| panic!("Bug: tool call at index {i} was never executed")))
        .collect())
}

async fn flush_read_only_batch(
    batch: &[(usize, &genai::chat::ToolCall)],
    tools: &[Box<dyn AgentTool>],
    event_tx: &tokio::sync::mpsc::UnboundedSender<AgentEvent>,
    cancel: &tokio_util::sync::CancellationToken,
    results: &mut [Option<genai::chat::ToolResponse>],
) -> anyhow::Result<()> {
    let futures: Vec<_> = batch
        .iter()
        .map(|(_, tc)| execute_single_tool(tc, tools, event_tx, cancel))
        .collect();

    let batch_results = futures::future::join_all(futures).await;

    for ((idx, _), result) in batch.iter().zip(batch_results) {
        results[*idx] = Some(result?);
    }

    Ok(())
}

async fn execute_single_tool(
    tool_call: &genai::chat::ToolCall,
    tools: &[Box<dyn AgentTool>],
    event_tx: &tokio::sync::mpsc::UnboundedSender<AgentEvent>,
    cancel: &tokio_util::sync::CancellationToken,
) -> anyhow::Result<genai::chat::ToolResponse> {
    // Emit start event
    event_tx.send(AgentEvent::ToolExecutionStart {
        call_id: tool_call.call_id.clone(),
        tool_name: tool_call.fn_name.clone(),
        arguments: tool_call.fn_arguments.clone(),
    })?;

    // Find the tool by name
    let tool = tools.iter().find(|t| t.name() == tool_call.fn_name);

    let (content, is_error) = match tool {
        Some(tool) => {
            tokio::select! {
                result = tool.execute(
                    &tool_call.call_id,
                    tool_call.fn_arguments.clone(),
                    event_tx,
                    cancel,
                ) => {
                    match result {
                        Ok(result) => (result, false),
                        Err(e) => (format!("Error executing tool: {e}"), true),
                    }
                }
                () = cancel.cancelled() => {
                    event_tx.send(AgentEvent::Aborted {
                        phase: "tool_execution".to_string(),
                        tool_call_id: Some(tool_call.call_id.clone()),
                    })?;
                    ("Tool execution aborted".to_string(), true)
                }
            }
        }
        None => (format!("Tool '{}' not found", tool_call.fn_name), true),
    };

    // Emit end event
    event_tx.send(AgentEvent::ToolExecutionEnd {
        call_id: tool_call.call_id.clone(),
        tool_name: tool_call.fn_name.clone(),
        result: content.clone(),
        is_error,
    })?;

    Ok(genai::chat::ToolResponse::new(
        tool_call.call_id.clone(),
        content,
    ))
}
