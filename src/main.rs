use anyhow::{Context, Ok, Result};
use astra_formats::{MessageBundle, TextBundle};
use gag::Gag;
use pathdiff::diff_paths;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;

use phf::phf_map;

static SUPPORTED_GAMEDATAS: phf::Map<&'static str, &'static str> = phf_map! {
    "person" => "Person",
    "skill" => "Skill",
    "shop" => "Shop",
    "item" => "Item",
    "god" => "God",
    "job" => "Job",
    "animset" => "AnimSet",
    "params" => "Params",
    "chapter" => "Chapter",
    "assettable" => "AssetTable",
    "animal" => "Animal",
    "calculator" => "Calculator",
    "cook" => "Cook",
    "achieve" => "Achieve",
    "reliance" => "Reliance",
};

use remove_empty_subdirs::remove_empty_subdirs;

use clap::{Args, Parser, Subcommand, ValueEnum};

fn create_required_directories(target_path: &str) -> Result<()> {
    fs::create_dir_all(Path::new(&target_path).join("patches/xml"))?;
    fs::create_dir_all(Path::new(&target_path).join("patches/msbt"))?;
    fs::create_dir_all(Path::new(&target_path).join("Data"))?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mod_path = cli.mod_path;
    match cli.command {
        Some(command) => match command {
            Commands::Convert => {
                convert_pre_cobalt_mod(mod_path)?;
            }
            Commands::Migrate(args) => match args.operation {
                Operation::msbt => {
                    println!("Migrating your MSBTs to plain text");
                    migrate_msbt(mod_path)?;
                }
            },
        },
        None => {
            // legacy action
            convert_pre_cobalt_mod(mod_path)?;
        }
    }

    Ok(())
}

fn convert_pre_cobalt_mod(mod_path: PathBuf) -> Result<()> {
    let romfs_path: PathBuf = mod_path.join("romfs");
    let mut target_path = mod_path.file_name().unwrap().to_str().unwrap().to_string();
    target_path.push_str(" (Cobalt)");
    let is_romfs: bool = romfs_path.is_dir();

    if !is_romfs {
        return Err(anyhow::anyhow!("The folder \"{}\" doesn't contain a \"romfs\" folder. Please make sure there is a folder named \"romfs\" in the folder.", mod_path.display()));
    }

    println!("Migrating your mod « {} »", &target_path);

    create_required_directories(&target_path)
        .with_context(|| "I had trouble creating the required directories for your mod.")?;

    for entry in WalkDir::new(Path::new(&mod_path).join("romfs"))
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let relative_path = diff_paths(path, &romfs_path).unwrap();
        if path.is_dir() {
            fs::create_dir_all(Path::new(&target_path).join(relative_path))
                .with_context(|| "I couldn't create the directory for your data files.")?;
            continue;
        }

        let file_name = entry.file_name().to_str().unwrap();

        // check if file is in SUPPORTED_GAMEDATAS
        // if it is, then we need to load it and do stuff
        // if it isn't, then we need to copy it over
        if file_name.ends_with(".xml.bundle") {
            let file_name = file_name.trim_end_matches(".xml.bundle");
            if let Some(&new_name) = SUPPORTED_GAMEDATAS.get(file_name) {
                convert_gamedata(&path.to_path_buf(), new_name, &target_path)?;
            } else {
                fs::copy(path, Path::new(&target_path).join(&relative_path))
                    .with_context(|| "I couldn't copy your gamedata bundle file.")?;
            };
        } else if file_name.ends_with(".bytes.bundle") {
            convert_msbt(&target_path, &relative_path, file_name, path)?;
        } else {
            fs::copy(path, Path::new(&target_path).join(&relative_path))
                .with_context(|| "I couldn't copy your gamedata bundle file.")?;
        }
    }

    {
        let _print_gag = Gag::stdout().unwrap();
        remove_empty_subdirs(Path::new(&target_path)).with_context(|| {
            "I ran into some problems cleaning up your mod. Please report this to the author. :). But your mod is probably fine."},
        )?;
    }
    println!("Done!");
    Ok(())
}

fn convert_msbt(
    target_path: &str,
    relative_path: &Path,
    file_name: &str,
    path: &Path,
) -> Result<()> {
    let mut locale_path = Path::new(&target_path)
        .join("patches")
        .join("msbt")
        .join("message")
        .join(
            relative_path
                .strip_prefix("Data/StreamingAssets/aa/Switch/fe_assets_message/")
                .unwrap(),
        );
    locale_path.pop();
    fs::create_dir_all(&locale_path)
        .with_context(|| "I couldn't create the directory for your message file.")?;
    let base_path = Path::new(&locale_path).join(file_name.strip_suffix(".bytes.bundle").unwrap());
    let mut bundle = MessageBundle::load(path).with_context(|| "Couldn't load message bundle")?;
    let script = bundle
        .take_script()
        .with_context(|| "Couldn't take script")?;
    let mut file = File::create(base_path.with_extension("txt")).with_context(|| {
        format!(
            "I couldn't create the file for your message file at {:?}",
            base_path
        )
    })?;
    file.write_all(script.as_bytes())
        .with_context(|| "I couldn't write your message file.")?;
    Ok(())
}

fn convert_gamedata(path: &PathBuf, new_name: &str, target_path: &str) -> Result<()> {
    let mut bundle = TextBundle::load(path).with_context(|| "Couldn't load text bundle")?;
    let raw = bundle
        .take_raw()
        .with_context(|| "Couldn't take raw bundle")?;
    let mut file = File::create(
        Path::new(&target_path)
            .join("patches")
            .join("xml")
            .join(new_name)
            .with_extension("xml"),
    )
    .with_context(|| format!("I couldn't create your gamedata file for {}", target_path))?;
    file.write_all(raw.as_slice())
        .with_context(|| format!("I couldn't write your gamedata file for {}", target_path))?;
    Ok(())
}

fn migrate_msbt(mod_path: PathBuf) -> Result<()> {
    for entry in WalkDir::new(&mod_path).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_dir() {
            continue;
        }
        let file_name = entry.file_name().to_str().unwrap();
        if file_name.ends_with("msbt") {
            let content = fs::read(entry.path()).with_context(|| {
                format!(
                    "I couldn't read your MSBT at location '{}'",
                    entry.path().display()
                )
            })?;
            let patch = astra_formats::MessageMap::from_slice(&content).with_context(|| {
                "I couldn't parse your MSBT. Please report this to the author. :)"
            })?;
            let wat = astra_formats::parse_msbt_script(&patch.messages)?;
            let mut file = File::create(entry.path().with_extension("txt")).with_context(|| {
                format!("I couldn't create your gamedata file for {}", file_name)
            })?;
            file.write_all(wat.as_bytes())
                .with_context(|| format!("I couldn't write your raw txt file for {}", file_name))?;
        }
    }
    println!("Done, check your mod folder for the new txt files.");
    Ok(())
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    mod_path: PathBuf,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert to Cobalt format
    Convert,
    /// Migrations for Cobalt deprecations
    Migrate(MigrateArgs),
}

#[derive(Args)]
struct MigrateArgs {
    operation: Operation,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Operation {
    /// Migrate MSBTs to plain text
    msbt,
}
