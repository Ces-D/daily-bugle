use anyhow::{Context, Result, bail};
use async_openai::{
    Client,
    types::{
        ChatCompletionRequestDeveloperMessage, ChatCompletionRequestDeveloperMessageContent,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestSystemMessageContent, CreateChatCompletionRequestArgs,
    },
};

const GIT_COMMIT_MESSAGE_SYSTEM_PROMPT:&str="
You are a commit message assistant following the Conventional Commits specification (v1.0.0). See: https://www.conventionalcommits.org/en/v1.0.0/

Given a unified git diff (the output of `git diff --cached` or similar), analyze the changes and generate a high-quality, conventional commit message.

Your output should:

1. Produce a **commit header** in the form:
   `<type>[optional scope][!]: <description>`

   - Choose the most appropriate **type** (e.g., feat, fix, docs, style, refactor, perf, test, chore).
   - Optionally include a **scope** if the diff clearly relates to a specific module or component.
   - If the changes introduce a breaking change, indicate it with `!` after the type or include a `BREAKING CHANGE:` footer.

2. Write a **description** that is:
   - Brief (one concise sentence).
   - In **imperative mood** (e.g., “add”, “fix”, “remove”, “update”).
   - Describes *what* was changed, not how.

3. Optionally include a **body** if needed:
   - Provide context for why the change was made.
   - Explain any non-obvious decisions or trade-offs.
   - Use multiple paragraphs if necessary.

4. Optionally include **footer(s)**:
   - Use `BREAKING CHANGE: …` if the commit introduces an API change or other backward-incompatible behavior.

5. Format:
   - One blank line between header and body.
   - One blank line between body and footer.
   - No line in header, body, or footer should exceed ~120 characters.

6. Be concise, but also sufficient to communicate intent to both humans and automation tools (e.g., for generating changelogs or version bumps).

---

Here is the diff:
";

pub async fn git_commit_message(model: &str) -> Result<String> {
    let git_diff_process = std::process::Command::new("git")
        .args(vec!["diff", "--staged", "':(exclude)*lock*'"])
        .output()?;
    if git_diff_process.status.success() {
        let client = Client::new();
        let diff = String::from_utf8_lossy(&git_diff_process.stdout);
        let request = CreateChatCompletionRequestArgs::default()
            .messages(vec![
                ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text(
                        GIT_COMMIT_MESSAGE_SYSTEM_PROMPT.to_string(),
                    ),
                    ..Default::default()
                }),
                ChatCompletionRequestMessage::Developer(ChatCompletionRequestDeveloperMessage {
                    content: ChatCompletionRequestDeveloperMessageContent::Text(diff.to_string()),
                    ..Default::default()
                }),
            ])
            .model(model)
            .build()
            .with_context(|| "Failed to create git diff chat completion request")?;
        let response = client
            .chat()
            .create(request)
            .await
            .with_context(|| "Failed to create chat completion")?;

        let commit_message = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .with_context(|| "No commit message in response")?;

        Ok(commit_message)
    } else {
        let error_message = String::from_utf8_lossy(&git_diff_process.stderr);
        bail!("Failed to get git diff: {}", error_message);
    }
}
