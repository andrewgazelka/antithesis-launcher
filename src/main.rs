use clap::Parser;
use reqwest::blocking::Client;
use serde_json::json;
use std::error::Error;
use std::process::Command;
use std::time::Duration;

fn get_git_email() -> String {
    Command::new("git")
        .args(["config", "--get", "user.email"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Antithesis API username
    #[arg(long, env = "ANTITHESIS_USERNAME")]
    username: String,

    /// Antithesis API password
    #[arg(long, env = "ANTITHESIS_PASSWORD")]
    password: String,

    /// Test duration in minutes
    #[arg(long, default_value = "15")]
    duration: String,

    /// Test description
    #[arg(long, default_value = "Basic test run")]
    description: String,

    /// Config image URL
    #[arg(long)]
    config_image: String,

    /// Email recipients (comma-separated)
    #[arg(long, default_value_t = get_git_email())]
    recipients: String,

    /// Docker images to test (can be specified multiple times)
    #[arg(long = "image")]
    images: Vec<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

    let mut params = json!({
        "antithesis.duration": args.duration,
        "antithesis.description": args.description,
        "antithesis.config_image": args.config_image,
        "antithesis.recipients": args.recipients,
    });

    if !args.images.is_empty() {
        let images = args.images.join(";");
        params
            .as_object_mut()
            .unwrap()
            .insert("antithesis.images".to_string(), json!(images));
    }

    println!("Launching experiment with parameters:");
    println!("{}", serde_json::to_string_pretty(&params)?);

    let response = client
        .post("https://hyperion.antithesis.com/api/v1/launch_experiment/basic_test")
        .basic_auth(args.username, Some(args.password))
        .json(&json!({ "params": params }))
        .send()?;

    response
        .error_for_status()
        .map(|_| println!("Successfully launched experiment"))
        .map_err(|e| format!("Failed to launch experiment: {e}").into())
}
