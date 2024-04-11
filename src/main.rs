/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-11
 */

use crate::app::App;
use args::{Args, Commands, FilterArgs};
use tui::event::Event;

mod app;
mod args;
mod gitignore_api;
mod tui;
mod util;

#[tokio::main]
async fn main() {
    let args: Args = clap::Parser::parse();
    let command = args.command.unwrap_or(Commands::Interactive);
    match command {
        Commands::List(args) => {
            print_templates(args).await;
        }
        Commands::Generate(args) => {
            print_gitignore(args.templates).await;
        }
        Commands::Interactive => {
            run_tui().await.expect("Unexpected error running TUI");
        }
    }
}

async fn run_tui() -> color_eyre::eyre::Result<()> {
    let mut tui = tui::Tui::new()
        .unwrap()
        .tick_rate(1.0)
        .frame_rate(30.0)
        .mouse(true);
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

async fn print_templates(args: FilterArgs) {
    match gitignore_api::get_template_names().await {
        Ok(mut templates) => {
            if let Some(filter) = args.filter {
                match regex::Regex::new(filter.as_str()) {
                    Err(error) => println!("Error: [regex] {}", error),
                    Ok(re) => {
                        templates = templates
                            .iter()
                            .filter(|t| re.is_match(t))
                            .map(|t| t.to_string())
                            .collect();
                    }
                }
            }
            if templates.is_empty() {
                println!("No templates to show");
            } else {
                for template in templates {
                    println!("{}", template);
                }
            }
        }
        Err(error) => eprintln!("Error: {}", error),
    }
}

async fn print_gitignore(template_names: Vec<String>) {
    let gt_result = gitignore_api::get_template(&template_names).await;
    match gt_result {
        Ok(result) => {
            println!("{}", result);
        }
        Err(error) => {
            println!(
                r#"Problem getting .gitignore for "{}": {}"#,
                template_names.join(" "),
                error
            )
        }
    };
}
