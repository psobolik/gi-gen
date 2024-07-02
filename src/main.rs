/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-11
 */

use args::{Args, Commands, FilterArgs};
use tui::event::Event;

use crate::app::App;

mod app;
mod args;
mod gitignore_api;
mod tui;
mod util;

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    let args: Args = clap::Parser::parse();
    let command = args.command.unwrap_or(Commands::Interactive);
    match command {
        Commands::List(args) => print_templates(args).await,
        Commands::Generate(args) => print_gitignore(args.templates).await,
        Commands::Interactive => run_tui().await,
    }
}

async fn run_tui() -> color_eyre::eyre::Result<()> {
    let mut tui = tui::Tui::new().unwrap().mouse(true);
    tui.enter()?;
    let mut app = App::default();
    app.set_event_tx(Some(tui.event_tx.clone()));

    loop {
        let event = tui.next().await?; // blocks until next event

        if let Event::Render = event.clone() {
            tui.draw(|f| {
                app.render(f);
            })?;
        }
        app.handle_event(event).await;
        if app.should_quit() {
            break;
        }
    }
    Ok(())
}

async fn print_templates(args: FilterArgs) -> color_eyre::eyre::Result<()> {
    let mut templates = gitignore_api::get_template_names().await?;
    if let Some(filter) = args.filter {
        let filter = regex::escape(filter.as_str());
        let re = regex::Regex::new(filter.as_str())?;
        templates = templates
            .iter()
            .filter_map(|t| {
                if re.is_match(t) {
                    Some(t.to_string())
                } else {
                    None
                }
            })
            .collect();
        if templates.is_empty() {
            println!(r#"No templates match "{}""#, filter);
        }
    }
    for template in templates {
        println!("{}", template);
    }
    Ok(())
}

async fn print_gitignore(template_names: Vec<String>) -> color_eyre::eyre::Result<()> {
    match gitignore_api::get_template(&template_names).await {
        Ok(result) => {
            println!("{}", result);
            Ok(())
        }
        Err(error) => {
            eprint!(
                r#"Problem getting .gitignore for "{}": "#,
                template_names.join(" ")
            );
            Err(color_eyre::eyre::Report::new(error))
        }
    }
}
