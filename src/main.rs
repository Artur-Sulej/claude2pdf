use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    path::PathBuf,
    process::Command,
};

use anyhow::Result;
use clap::Parser as ClapParser;
use pulldown_cmark::{html, Options, Parser};
use regex::Regex;
use serde::Deserialize;
use syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet};

/// Convert Claude Code JSONL conversations to syntax-highlighted PDFs.
#[derive(ClapParser)]
#[command(version, about)]
struct Cli {
    /// Path to the input JSONL file
    input: PathBuf,

    /// Path for the output PDF (defaults to <input>.pdf)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct Root {
    #[serde(rename = "type")]
    record_type: Option<String>,
    message: Option<Message>,
}

#[derive(Debug, Deserialize)]
struct Message {
    role: String,
    content: Content,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Content {
    String(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let pdf_file = cli
        .output
        .unwrap_or_else(|| cli.input.with_extension("pdf"));
    let html_file = pdf_file.with_extension("html");

    let markdown = extract_conversation_markdown(&cli.input)?;
    let html_content = render_markdown_with_highlighting(&markdown)?;

    // We need absolute path for Chrome to work reliably with file://
    let abs_html_file = std::fs::canonicalize(std::env::current_dir()?)?.join(&html_file);
    std::fs::write(&html_file, html_content)?;
    
    render_pdf(&abs_html_file, &pdf_file)?;

    Ok(())
}

fn extract_conversation_markdown(path: &Path) -> Result<String> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut output = String::new();

    for line in reader.lines() {
        let line = line?;
        let parsed: Root = serde_json::from_str(&line)?;

        if parsed.record_type.as_deref() != Some("assistant")
            && parsed.record_type.as_deref() != Some("user")
        {
            continue;
        }

        let message = match parsed.message {
            Some(m) => m,
            None => continue,
        };

        output.push_str(&format!("## {}\n\n", message.role));

        match message.content {
            Content::String(text) => {
                output.push_str(&text);
                output.push_str("\n\n");
            }
            Content::Blocks(blocks) => {
                for block in blocks {
                    if block.block_type != "text" {
                        continue;
                    }

                    if let Some(text) = block.text {
                        output.push_str(&text);
                        output.push_str("\n\n");
                    }
                }
            }
        }
    }

    Ok(output)
}

fn render_markdown_with_highlighting(md: &str) -> Result<String> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    let code_block_re = Regex::new(r"(?s)```(\w+)?\n(.*?)```")?;

    let highlighted = code_block_re.replace_all(md, |caps: &regex::Captures| {
        let lang = caps.get(1).map(|m| m.as_str()).unwrap_or("txt");
        let code = caps.get(2).unwrap().as_str();

        let syntax = ps
            .find_syntax_by_token(lang)
            .unwrap_or_else(|| ps.find_syntax_plain_text());

        highlighted_html_for_string(code, &ps, syntax, theme)
            .unwrap_or_else(|_| format!("<pre><code>{}</code></pre>", code))
    });

    let mut html_output = String::new();
    let parser = Parser::new_ext(&highlighted, Options::all());
    html::push_html(&mut html_output, parser);

    Ok(format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
body {{ font-family: Arial, sans-serif; padding: 40px; }}
pre {{ overflow-x: auto; background-color: #2b303b; padding: 15px; border-radius: 5px; }}
code {{ font-family: monospace; }}
h2 {{ border-bottom: 1px solid #ddd; padding-bottom: 4px; }}
</style>
</head>
<body>
{}
</body>
</html>"#,
        html_output
    ))
}

fn render_pdf(html: &Path, pdf: &Path) -> Result<()> {
    let chrome_path = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
    
    let status = Command::new(chrome_path)
        .arg("--headless")
        .arg("--disable-gpu")
        .arg(format!("--print-to-pdf={}", pdf.display()))
        .arg(format!("file://{}", html.display()))
        .status()?;

    if !status.success() {
        anyhow::bail!("Chrome failed to generate PDF");
    }

    Ok(())
}
