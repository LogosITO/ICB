use clap::Parser;
use icb_common::Language;
use icb_core::parser::manager::ParserManager;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "icb")]
#[command(about = "Infinite Code Blueprint CLI")]
struct Cli {
    path: PathBuf,
    #[arg(short, long)]
    language: String,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let lang = match args.language.as_str() {
        "python" => Language::Python,
        "rust" => Language::Rust,
        "javascript" => Language::JavaScript,
        _ => anyhow::bail!("Неподдерживаемый язык: {}", args.language),
    };

    let source = fs::read_to_string(&args.path)?;

    let manager = ParserManager::new()?;

    let tree = manager.parse(lang, &source)?;
    println!(
        "✅ Парсинг успешен! Корневой узел: {:?}",
        tree.root_node().kind()
    );

    Ok(())
}
