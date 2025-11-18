use anyhow::{Context, Result, bail};
use log::{info, trace};
use std::path::PathBuf;

#[derive(Debug, Default, Clone)]
struct Section {
    heading: Option<String>,
    content: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Resume {
    name: Option<String>,
    sections: Vec<Section>,
}

impl Resume {
    pub fn to_complete_string(&self) -> String {
        // Format and return the resume
        let mut result = String::new();
        if let Some(name) = &self.name {
            result.push_str(&format!("Name: {}\n\n", name));
        }

        for section in &self.sections {
            if let Some(heading) = &section.heading {
                result.push_str(&format!("{}\n", heading));
            }
            for content in &section.content {
                result.push_str(&format!("{}\n", content));
            }
            result.push('\n');
        }
        result
    }
}

/// Formatting Assumptions:
/// - The 3rd line of the resume is the name -> first 2 are blank lines, often used for formatting
/// - The headings param is a list of strings that are used to find the relevant sections of the
/// resume. The order of the headings matches the order of the sections in the resume.
/// - Relevant sections are separated from each other by an empty line.
///
/// WARNING: The parser may not be 100% accurate. It is possible that it will incorrectly identify
/// sections or headings. Please double-check the extracted information using RUST_LOG=trace.
pub fn extract_resume_information(pdf: &PathBuf, headings: Vec<String>) -> Result<Resume> {
    let extracted_text =
        pdf_extract::extract_text(pdf).with_context(|| "Failed to extract text from pdf")?;

    let mut headings_index = 0;
    let mut resume = Resume::default();
    let mut current_section: Option<Section> = None;
    let mut current_content = String::new();

    for (index, line) in extracted_text.lines().enumerate() {
        trace!("Line {}: {}", index, line);

        // First line is the name
        if index == 2 {
            let name = line.trim();
            if name.is_empty() {
                bail!("Invalid resume format: first line is empty");
            } else {
                resume.name = Some(name.to_string());
            }
            continue;
        }

        // Check if this line matches the next expected heading
        let is_heading =
            headings_index < headings.len() && line.trim().starts_with(&headings[headings_index]);

        if is_heading {
            info!("Found heading: {}", line);
            // Save accumulated content to current section before switching
            if !current_content.is_empty() {
                if let Some(ref mut section) = current_section {
                    section.content.push(current_content.clone());
                    current_content = String::new();
                }
            }

            // Save current section to resume
            if let Some(section) = current_section.take() {
                trace!("Saving section: {:?}", section);
                resume.sections.push(section);
            }

            // Start new section
            trace!("Starting new section: {}", headings[headings_index].clone());
            let mut new_section = Section::default();
            new_section.heading = Some(headings[headings_index].clone());
            current_section = Some(new_section);
            headings_index += 1;
            continue;
        }

        // Handle content lines
        if line.trim().is_empty() {
            // Empty line - push accumulated content to current section
            if !current_content.is_empty() {
                if let Some(ref mut section) = current_section {
                    section.content.push(current_content.clone());
                    current_content = String::new();
                }
            }
        } else {
            // Non-empty line - accumulate content
            if current_section.is_some() {
                if !current_content.is_empty() {
                    current_content.push(' ');
                }
                current_content.push_str(line.trim());
            }
        }
    }

    // Push any remaining content
    if !current_content.is_empty() {
        if let Some(ref mut section) = current_section {
            section.content.push(current_content);
        }
    }

    // Push final section
    if let Some(section) = current_section {
        resume.sections.push(section);
    }

    Ok(resume)
}
