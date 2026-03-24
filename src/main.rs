use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, Write};
use std::process::Command;

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

fn detect_shell() -> String {
    env::var("SHELL")
        .unwrap_or_else(|_| String::from("/bin/bash"))
        .rsplit('/')
        .next()
        .unwrap_or("bash")
        .to_string()
}

fn detect_os() -> String {
    let os = env::consts::OS;
    // Try to get more detail on Linux
    if os == "linux" {
        if let Ok(contents) = std::fs::read_to_string("/etc/os-release") {
            for line in contents.lines() {
                if line.starts_with("PRETTY_NAME=") {
                    return line
                        .trim_start_matches("PRETTY_NAME=")
                        .trim_matches('"')
                        .to_string();
                }
            }
        }
    }
    os.to_string()
}

fn build_prompt(query: &str, os: &str, shell: &str) -> String {
    format!(
        "You are a terminal command generator. The user describes what they want to do in natural language. \
         You respond with ONLY the exact terminal command — nothing else. No explanation, no markdown, no backticks, no newline before or after. \
         Just the raw command.\n\n\
         OS: {os}\n\
         Shell: {shell}\n\
         Working directory: {cwd}\n\n\
         User request: {query}",
        os = os,
        shell = shell,
        cwd = env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| ".".into()),
        query = query,
    )
}

fn call_llm(query: &str) -> Result<String, String> {
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY not set. Export it first:\n  export OPENAI_API_KEY=sk-...")?;

    let base_url = env::var("OPENAI_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

    let model = env::var("CTRLR_MODEL")
        .unwrap_or_else(|_| "gpt-4o-mini".to_string());

    let os = detect_os();
    let shell = detect_shell();
    let system_prompt = build_prompt(query, &os, &shell);

    let request_body = ChatRequest {
        model,
        messages: vec![
            Message {
                role: "system".into(),
                content: system_prompt,
            },
            Message {
                role: "user".into(),
                content: query.to_string(),
            },
        ],
        temperature: 0.0,
        max_tokens: 256,
    };

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let body_json = serde_json::to_string(&request_body).unwrap();

    let resp = minreq::post(&url)
        .with_header("Authorization", format!("Bearer {}", api_key))
        .with_header("Content-Type", "application/json")
        .with_body(body_json)
        .send()
        .map_err(|e| format!("API request failed: {}", e))?;

    if resp.status_code != 200 {
        return Err(format!(
            "API returned status {}: {}",
            resp.status_code,
            resp.as_str().unwrap_or("unknown error")
        ));
    }

    let body: ChatResponse = serde_json::from_str(resp.as_str().unwrap_or(""))
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    body.choices
        .first()
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| "No response from API".into())
}

fn run_command(cmd: &str) {
    let shell = detect_shell();
    let status = Command::new(&shell)
        .arg("-c")
        .arg(cmd)
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!("\x1b[33mCommand exited with status: {}\x1b[0m", s);
        }
        Err(e) => {
            eprintln!("\x1b[31mFailed to execute command: {}\x1b[0m", e);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("Usage: nli <natural language command>");
        eprintln!("  e.g: nli list files in descending order of size");
        std::process::exit(1);
    }

    let query = args.join(" ");

    // Call LLM
    let command = match call_llm(&query) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("\x1b[31m{}\x1b[0m", e);
            std::process::exit(1);
        }
    };

    // Display the command
    println!("\n  \x1b[1;36m❯\x1b[0m \x1b[1m{}\x1b[0m\n", command);

    // Ask to run
    print!("  Run? \x1b[2m(Y/n)\x1b[0m ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_lowercase();

    if input.is_empty() || input == "y" || input == "yes" {
        println!();
        run_command(&command);
    }
}
