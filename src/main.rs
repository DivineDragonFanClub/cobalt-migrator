#![feature(lazy_cell)]

use astra_formats::TextBundle;
use gag::Gag;
use pathdiff::diff_paths;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;

use std::collections::HashMap;

use std::sync::LazyLock;

static SUPPORTED_GAMEDATAS: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    HashMap::from([
        ("person", "Person"),
        ("skill", "Skill"),
        ("shop", "Shop"),
        ("item", "Item"),
        ("god", "God"),
        ("job", "Job"),
        ("animset", "AnimSet"),
        ("params", "Params"),
        ("chapter", "Chapter"),
        ("assettable", "AssetTable"),
        ("animal", "Animal"),
        ("calculator", "Calculator"),
        ("reliance", "Reliance"),
    ])
});

use remove_empty_subdirs::remove_empty_subdirs;

use clap::Parser;

fn create_required_directories(target_path: &str) -> std::io::Result<()> {
    fs::create_dir_all(Path::new(&target_path).join("patches/xml"))?;
    fs::create_dir_all(Path::new(&target_path).join("patches/msbt"))?;
    fs::create_dir_all(Path::new(&target_path).join("Data"))?;
    Ok(())
}

fn main() {
    let cli = Args::parse();
    let mod_path = cli.mod_path;
    let romfs_path: PathBuf = mod_path.join("romfs");
    let mut target_path = mod_path.file_name().unwrap().to_str().unwrap().to_string();
    target_path.push_str(" (Cobalt)");
    let is_romfs: bool = romfs_path.is_dir();

    if !is_romfs {
        println!("The folder \"{}\" doesn't contain a \"romfs\" folder. Please make sure there is a folder named \"romfs\" in the folder.", mod_path.display());
        return;
    }

    println!("Migrating your mod « {} »", &target_path);

    create_required_directories(&target_path).expect(
        "I had trouble creating the required directories. Please report this to the author.",
    );

    for entry in WalkDir::new(Path::new(&mod_path).join("romfs"))
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let relative_path = diff_paths(path, &romfs_path).unwrap();
        if path.is_dir() {
            fs::create_dir_all(Path::new(&target_path).join(relative_path)).expect("I couldn't create the directory for your data files. Please report this to the author.");
            continue;
        }

        let file_name = entry.file_name().to_str().unwrap();
        // check if file is in SUPPORTED_GAMEDATAS
        // if it is, then we need to load it and do stuff
        // if it isn't, then we need to copy it over
        if file_name.ends_with(".xml.bundle") {
            let file_name = file_name.trim_end_matches(".xml.bundle");
            if let Some(&new_name) = SUPPORTED_GAMEDATAS.get(file_name) {
                migrate_gamedata(&path.to_path_buf(), new_name, &target_path);
            } else {
                fs::copy(path, Path::new(&target_path).join(&relative_path)).expect(
                    "I couldn't copy your gamedata bundle file. Please report this to the author.",
                );
            };
        } else if file_name.ends_with(".bytes.bundle") {
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
            fs::create_dir_all(&locale_path).expect("I couldn't create the directory for your message file. Please report this to the author.");

            let my_message = TextBundle::load(path);
            match my_message {
                Ok(mut message) => {
                    let my_result = message.take_raw();
                    let mut file = File::create(
                        Path::new(&locale_path)
                            .join(file_name.strip_suffix(".bytes.bundle").unwrap())
                            .with_extension("msbt"),
                    )
                    .unwrap();
                    file.write_all(my_result.unwrap().as_slice()).expect(
                        "I couldn't write your message file. Please report this to the author.",
                    );
                }
                Err(e) => {
                    println!(
                        "Error loading message: {:?}. Please report this to the author.",
                        e
                    );
                }
            }
        } else {
            fs::copy(path, Path::new(&target_path).join(&relative_path)).expect(
                "I couldn't copy your gamedata bundle file. Please report this to the author.",
            );
        }
    }

    {
        let _print_gag = Gag::stdout().unwrap();
        remove_empty_subdirs(Path::new(&target_path)).expect(
            "I ran into some problems cleaning up your mod. Please report this to the author. :). But your mod is probably fine.",
        );
    }
    println!("Done!");
}

fn migrate_gamedata(path: &PathBuf, new_name: &str, target_path: &str) {
    let my_bundle = TextBundle::load(path);

    match my_bundle {
        Ok(mut bundle) => {
            let my_result = bundle.take_raw().unwrap();
            // println!("Bundle: {:?}", my_result);
            let mut file = File::create(
                Path::new(target_path)
                    .join("patches")
                    .join("xml")
                    .join(new_name)
                    .with_extension("xml"),
            )
            .unwrap();
            file.write_all(my_result.as_slice())
                .expect("I couldn't write your gamedata file. Please report this to the author.");
        }
        Err(e) => {
            println!("Error loading bundle: {:?}", e);
        }
    }
}

/// Simple program to migrate a mod
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    mod_path: PathBuf,
}
