use anyhow::{Context, Result, bail};
use async_openai::{
    Client,
    types::{
        ChatCompletionRequestDeveloperMessage, ChatCompletionRequestDeveloperMessageContent,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestSystemMessageContent, CreateChatCompletionRequest,
        CreateChatCompletionRequestArgs,
    },
};

fn system_message(text: &str) -> ChatCompletionRequestMessage {
    ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
        content: ChatCompletionRequestSystemMessageContent::Text(text.to_string()),
        ..Default::default()
    })
}

fn developer_message(text: String) -> ChatCompletionRequestMessage {
    ChatCompletionRequestMessage::Developer(ChatCompletionRequestDeveloperMessage {
        content: ChatCompletionRequestDeveloperMessageContent::Text(text),
        ..Default::default()
    })
}

async fn make_chat_completion_request(request: CreateChatCompletionRequest) -> Result<String> {
    let client = Client::new();
    let response = client
        .chat()
        .create(request)
        .await
        .with_context(|| "Failed to create chat completion")?;
    let message = response
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone())
        .with_context(|| "Failed to gather first message from response")?;
    Ok(message)
}

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
        .args(vec!["diff", "--staged", ":(exclude)*lock*"])
        .output()?;
    if git_diff_process.status.success() {
        let diff = String::from_utf8_lossy(&git_diff_process.stdout);
        if diff.is_empty() {
            bail!("No changes detected in git diff");
        };
        let request = CreateChatCompletionRequestArgs::default()
            .messages(vec![
                system_message(GIT_COMMIT_MESSAGE_SYSTEM_PROMPT),
                developer_message(diff.to_string()),
            ])
            .model(model)
            .build()
            .with_context(|| "Failed to create git diff chat completion request")?;
        let commit_message = make_chat_completion_request(request).await?;
        Ok(commit_message)
    } else {
        let error_message = String::from_utf8_lossy(&git_diff_process.stderr);
        bail!("Failed to get git diff: {}", error_message);
    }
}

const PULL_REQUEST_MESSAGE_SYSTEM_PROMPT: &str = "
You are a Pull Request description assistant.

Given a list of commits (typically all commits ahead of `origin/main` or another target branch), analyze the changes
and generate a high-quality Pull Request **title** and **description** suitable for engineering teams, code review, and
changelog automation.

Your output should:

1. Produce a **Pull Request title**:
   - Short (ideally < 80 characters).
   - Written in imperative mood (“add”, “update”, “fix”, “refactor”).
   - Summarize the overall impact of all commits.
   - If the changes are broad, prefer describing the highest-level impact rather than listing everything.

2. Produce a **Pull Request description** containing:
   - A concise **summary** of what was changed and why.
   - A **Changes** section that lists key modifications (group similar changes together).
   - A **Motivation** or **Context** section when relevant—explain why the change set is needed.
   - A **Technical Notes** section for non-obvious implementation decisions, architectural impacts, trade-offs, or
     constraints.
   - A **Breaking Changes** section if any commit introduces backward-incompatible behavior.
   - A **Testing** section describing how the changes were tested (unit tests, manual steps, screenshots, etc.).
   - A **Checklist** section for standard PR hygiene (optional but encouraged).

3. Follow formatting guidelines:
   - Use clear Markdown headings.
   - Keep line lengths ~120 characters or less.
   - Prefer bullet points for changes and technical notes.
   - Do not simply restate commit messages—synthesize them into a unified narrative.

4. The description should be useful for:
   - Reviewers trying to understand intent and scope.
   - Future developers reading the commit history.
   - Release note or changelog generation.

---

You will be provided with:  
- A list of commits ahead of the target branch  
Use this information to generate the PR title and description.
";

pub async fn git_pull_request_message(model: &str) -> Result<String> {
    let commits_up_to_main_branch = std::process::Command::new("git")
        .args(vec!["log", "origin/main.."])
        .output()?;
    if commits_up_to_main_branch.status.success() {
        let diff = String::from_utf8_lossy(&commits_up_to_main_branch.stdout);
        if diff.is_empty() {
            bail!("No changes detected in git diff");
        };
        let request = CreateChatCompletionRequestArgs::default()
            .messages(vec![
                system_message(PULL_REQUEST_MESSAGE_SYSTEM_PROMPT),
                developer_message(diff.to_string()),
            ])
            .model(model)
            .build()
            .with_context(|| "Failed to create git pull request completion request")?;
        let pull_request_message = make_chat_completion_request(request).await?;
        Ok(pull_request_message)
    } else {
        let error_message = String::from_utf8_lossy(&commits_up_to_main_branch.stderr);
        bail!("Failed to get commits up to main branch: {}", error_message);
    }
}
