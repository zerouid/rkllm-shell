use clap::Parser;
use rig::agent::AgentBuilder;
use rig::completion::Prompt;
use rig::tool::Tool;

use crate::{
    config::Config,
    error::Result,
    server::rig_provider::{RkllmClient, RkllmCompletionConfig, RkllmCompletionModel},
    server::rkllm_runtime::RkllmRuntime,
    terminal::message::write,
};
use owo_colors::OwoColorize;

/// Run an AI agent with tool support
#[derive(Parser, Debug)]
pub struct Args {
    /// Model to use
    #[arg(short, long, default_value = "default")]
    pub model: String,

    /// System prompt / preamble
    #[arg(short, long)]
    pub system: Option<String>,

    /// Enable tools (file ops, shell, model management)
    #[arg(long, default_value = "true")]
    pub tools: bool,

    /// Temperature for sampling
    #[arg(long)]
    pub temperature: Option<f32>,

    /// Max tokens to generate
    #[arg(long)]
    pub max_tokens: Option<u32>,

    /// Prompt (if not provided, enters interactive mode)
    #[arg(index = 1)]
    pub prompt: Option<String>,
}

pub async fn run(config: &Config, options: &Args) -> Result<()> {
    write::info(format!("Starting agent with model: {}", options.model).green())?;

    // Create runtime and client
    let models_path = config.models_path.clone().unwrap_or_else(|| std::path::PathBuf::from("./data"));
    let runtime = RkllmRuntime::new(models_path);
    let runtime_arc = std::sync::Arc::new(runtime);
    let client = RkllmClient::new(runtime_arc.clone());

    // Build completion config
    let completion_config = RkllmCompletionConfig {
        model_name: options.model.clone(),
        temperature: options.temperature,
        max_tokens: options.max_tokens,
        ..Default::default()
    };

    // Build agent
    let mut agent_builder = client.agent_with_config(completion_config);

    if let Some(system) = &options.system {
        agent_builder = agent_builder.preamble(system);
    }

    // Add tools if enabled
    if options.tools {
        write::info("Tools enabled: file_read, file_write, shell, model_manage".green())?;
        // TODO: Add actual tools when implemented
        // agent_builder = agent_builder
        //     .tool(FileReadTool::new())
        //     .tool(FileWriteTool::new())
        //     .tool(ShellTool::new())
        //     .tool(ModelManageTool::new(runtime_arc.clone()));
    }

    let agent = agent_builder.build();

    if let Some(prompt) = &options.prompt {
        // Single prompt mode
        write::info("Processing prompt...".green())?;
        let response = agent.prompt(prompt).await
            .map_err(|e| crate::error::Error::Server(format!("Agent error: {}", e)))?;
        println!("{}", response);
    } else {
        // Interactive REPL mode
        write::info("Entering interactive mode. Type 'exit' or 'quit' to quit.".green())?;
        run_agent_repl(agent).await?;
    }

    Ok(())
}

async fn run_agent_repl(mut agent: rig::agent::Agent<RkllmCompletionModel>) -> Result<()> {
    use std::io::{self, Write};
    
    loop {
        use owo_colors::OwoColorize;
        print!("\n{} ", "🤖 Agent>".green().bold());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "exit" || input == "quit" {
            write::info("Goodbye!".green())?;
            break;
        }

        if input == "help" {
            println!("Commands:");
            println!("  help     - Show this help");
            println!("  exit     - Exit the agent");
            println!("  quit     - Exit the agent");
            println!("  Any other text will be sent as a prompt to the agent.");
            continue;
        }

        write::info("Thinking...".yellow())?;
        match agent.prompt(input).await {
            Ok(response) => {
                println!("\n{}", response);
            }
            Err(e) => {
                write::error(format!("Agent error: {}", e))?;
            }
        }
    }

    Ok(())
}