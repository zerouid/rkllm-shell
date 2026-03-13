use std::sync::{Arc, Mutex};
use std::time::Duration;

use clap::Subcommand;
use clap_repl::reedline::{
    DefaultPrompt, DefaultPromptSegment, FileBackedHistory,
};
use clap_repl::{ClapEditor, ReadCommandOutput};
use tokio::sync::oneshot;
use tokio::time::sleep;

use crate::args::Args;
use crate::terminal::color::Colorize;
use crate::terminal::message::write;

pub mod serve;
pub mod create;
pub mod show;
pub mod run;
pub mod stop;
pub mod quit;
pub mod pull;
pub mod push;
pub mod list;
pub mod ps;
pub mod cp;
pub mod rm;
pub mod info;

/*
Ollama CLI
serve       Start ollama
create      Create a model from a Modelfile
show        Show information for a model
run         Run a model
stop        Stop a running model
pull        Pull a model from a registry
push        Push a model to a registry
list        List models
ps          List running models
cp          Copy a model
rm          Remove a model
help        Help about any command
 */

#[derive(Subcommand)]
pub enum Command {
    Serve(serve::Args),
    Info(info::Args),
    Show(show::Args),
    Create(create::Args),
    Run(run::Args),
    Stop(stop::Args),
    Quit(quit::Args),
    Pull(pull::Args),
    Push(push::Args),
    List(list::Args),
    Ps(ps::Args),
    Cp(cp::Args),
    Rm(rm::Args),
}

impl Command {
    pub async fn run(self, config: &crate::config::Config, shutdown_tx: Option<oneshot::Sender<()>>) -> crate::error::Result<()> {
        match self {
            Command::Serve(args) => serve::run(config, &args).await,
            Command::Info(_) => info::run(config).await,
            Command::Show(args) => show::run(config, &args).await,
            Command::Create(args) => create::run(config, &args).await,
            Command::Run(args) => run::run(config, &args).await,
            Command::Stop(args) => { 
                stop::run(config, &args).await?;
                write::info("Server shutdown initiated")?;
                if let Some(tx) = shutdown_tx {
                    let _ = tx.send(());
                }
                sleep(Duration::from_millis(100)).await;
                Ok(())
             },
            Command::Quit(args) => quit::run(config, &args).await,
            Command::Pull(args) => pull::run(config, &args).await,
            Command::Push(args) => push::run(config, &args).await,
            Command::List(args) => list::run(config, &args).await,
            Command::Ps(args) => ps::run(config, &args).await,
            Command::Cp(args) => cp::run(config, &args).await,
            Command::Rm(args) => rm::run(config, &args).await,
        }
    }
}

pub async fn run_repl(config: &crate::config::Config, shutdown_tx: oneshot::Sender<()>) -> crate::error::Result<()> {
    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Basic("rkllm-shell".to_owned()),
        ..DefaultPrompt::default()
    };
    let history_path = config.dir.join(".history");
    let mut rl = ClapEditor::<Args>::builder()
        .with_prompt(Box::new(prompt))
        .with_editor_hook(move |reed| {
            // Do custom things with `Reedline` instance here
            reed.with_history(Box::new(
                FileBackedHistory::with_file(10000, history_path.clone())
                    .unwrap(),
            ))
        })
        .build();
    let shutdown_tx = Arc::new(Mutex::new(Some(shutdown_tx)));
    loop {
        match rl.read_command() {
            ReadCommandOutput::Command(args) => {
                if let Some(cmd) = args.command {
                    let is_stop_command = matches!(cmd, Command::Stop(_));
                    let is_quit_command = matches!(cmd, Command::Quit(_));
                    let tx = if is_stop_command {
                            shutdown_tx.lock().unwrap().take()
                    } else {
                        None
                    };

                    if let Err(e) = cmd.run(config, tx).await {
                        println!("Error executing command: {}", e);
                    }

                    if is_stop_command || is_quit_command {
                        break;
                    }
                }
            }
            ReadCommandOutput::EmptyLine => (),
            ReadCommandOutput::ClapError(e) => {
                e.print().unwrap();
            }
            ReadCommandOutput::ShlexError => {
                write::error("Error: input was not valid and could not be processed".red())?;
            }
            ReadCommandOutput::ReedlineError(e) => {
                panic!("{e}");
            }
            ReadCommandOutput::CtrlC => continue,
            ReadCommandOutput::CtrlD => break,
        }
    }
    Ok(())
}
