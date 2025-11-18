mod resume;

use anyhow::{Context, Result, ensure};
use async_openai::{
    Client,
    types::{
        ChatCompletionRequestDeveloperMessage, ChatCompletionRequestDeveloperMessageContent,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestSystemMessageContent, CreateChatCompletionRequestArgs,
    },
};
use std::path::PathBuf;

const COVER_LETTER_SYSTEM_PROMPT: &str = "
Given three inputs— the user’s résumé, their personal/work profile, and the job description. Create a concise, compelling cover letter tailored to the specific company and role.

The cover letter must:

Show genuine interest in that company and that role, referencing what makes the company appealing.
Demonstrate strong technical alignment to the job’s requirements.
Reflect the user’s personality, values, and working style.
Clearly answer: “What can I bring to the company that others cannot?” by highlighting distinctive achievements, unique combinations of skills, or uncommon perspectives.
Be confident, and clear.
Convert resume details into narrative impact rather than repeating bullet points.
Write the cover letter in natural paragraphs, professional but warm, and focused on showing why the candidate is a uniquely strong fit.
";

pub async fn generate_cover_letter(
    resume_path: &PathBuf,
    resume_headings: Vec<String>,
    job_path: &PathBuf,
    profile_path: &PathBuf,
    model: &str,
) -> Result<String> {
    ensure!(
        resume_path.exists()
            && resume_path.is_file()
            && resume_path.extension() == Some("pdf".as_ref()),
        format!("Invalid resume path: {}", resume_path.display())
    );
    ensure!(
        profile_path.exists()
            && profile_path.is_file()
            && profile_path.extension() == Some("txt".as_ref()),
        format!("Invalid profile path: {}", profile_path.display())
    );
    ensure!(
        job_path.exists() && job_path.is_file() && job_path.extension() == Some("txt".as_ref()),
        format!("Invalid job path: {}", job_path.display())
    );
    let resume_data = resume::extract_resume_information(resume_path, resume_headings)?;
    let profile = std::fs::read_to_string(profile_path)?.trim().to_string();
    let job = std::fs::read_to_string(job_path)?.trim().to_string();

    let client = Client::new();
    let request = CreateChatCompletionRequestArgs::default()
        .messages(vec![
            ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::Text(
                    COVER_LETTER_SYSTEM_PROMPT.to_string(),
                ),
                ..Default::default()
            }),
            ChatCompletionRequestMessage::Developer(ChatCompletionRequestDeveloperMessage {
                content: ChatCompletionRequestDeveloperMessageContent::Text(format!(
                    "<resume>{}</resume><profile>{}</profile><job>{}</job>",
                    resume_data.to_complete_string(),
                    profile,
                    job
                )),
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
    let cover_letter = response
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone())
        .with_context(|| "No cover letter in response")?;

    Ok(cover_letter)
}

const GENERATE_USER_SUMMARY_SYSTEM_PROMPT: &str = "
You are generating a concise professional summary for a résumé. You will be provided the candidate’s resume, a user profile, and the job description.
Your task is to produce a 2–3 sentence summary that:

Reflects the candidate’s role, expertise, and top strengths

Clearly shows why the candidate is relevant to the role

Uses phrasing that is as close as reasonably possible to the job description’s language

Stays short, specific, and professional

Avoids fluff, clichés, and generic statements

Only output the completed professional summary.
";

pub async fn generate_professional_summary(
    resume_path: &PathBuf,
    resume_headings: Vec<String>,
    job_path: &PathBuf,
    profile_path: &PathBuf,
    model: &str,
) -> Result<String> {
    ensure!(
        resume_path.exists()
            && resume_path.is_file()
            && resume_path.extension() == Some("pdf".as_ref()),
        "Invalid resume path"
    );
    ensure!(
        profile_path.exists()
            && profile_path.is_file()
            && profile_path.extension() == Some("txt".as_ref()),
        "Invalid profile path"
    );
    ensure!(
        job_path.exists() && job_path.is_file() && job_path.extension() == Some("txt".as_ref()),
        "Invalid job path"
    );
    let resume_data = resume::extract_resume_information(resume_path, resume_headings)?;
    let profile = std::fs::read_to_string(profile_path)?.trim().to_string();
    let job = std::fs::read_to_string(job_path)?.trim().to_string();

    let client = Client::new();
    let request = CreateChatCompletionRequestArgs::default()
        .messages(vec![
            ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::Text(
                    GENERATE_USER_SUMMARY_SYSTEM_PROMPT.to_string(),
                ),
                ..Default::default()
            }),
            ChatCompletionRequestMessage::Developer(ChatCompletionRequestDeveloperMessage {
                content: ChatCompletionRequestDeveloperMessageContent::Text(format!(
                    "<resume>{}</resume><profile>{}</profile><job>{}</job>",
                    resume_data.to_complete_string(),
                    profile,
                    job
                )),
                ..Default::default()
            }),
        ])
        .model(model)
        .build()
        .with_context(|| "Failed to create professional summary chat completion request")?;
    let response = client
        .chat()
        .create(request)
        .await
        .with_context(|| "Failed to create chat completion")?;
    let summary = response
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone())
        .with_context(|| "No professional summary in response")?;

    Ok(summary)
}
