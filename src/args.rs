/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-10
 */

#[derive(Debug, clap::Args)]
pub(super) struct FilterArgs {
    /// Filter (regular expression) to apply to list of templates
    pub(super) filter: Option<String>,
}

#[derive(Debug, clap::Args)]
pub(super) struct TemplateArgs {
    /// One or more gitignore templates
    #[arg(name = "template", required = true)]
    pub(super) templates: Vec<String>,
}

#[derive(Debug, clap::Subcommand)]
pub(super) enum Commands {
    /// List available templates with optional filter applied
    List(FilterArgs),
    /// Generate .gitignore using specified template(s)
    Generate(TemplateArgs),
    /// Pick templates interactively and generate .gitignore (default)
    Interactive,
}

#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
pub(super) struct Args {
    /// Optional subcommand
    #[clap(subcommand)]
    pub(super) command: Option<Commands>,
}
