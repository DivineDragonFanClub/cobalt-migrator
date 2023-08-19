use astra_formats::{Book, MessageBundle, TextBundle};
use astra_formats::{Bundle, BundleFile};
use pathdiff::diff_paths;
use phf::phf_map;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;

use clap::Parser;

const GAMEDATA_PATH: &str = "romfs/Data/StreamingAssets/aa/Switch/fe_assets_gamedata";

const BROWSERS: &[&str] = &["firefox", "chrome"];

// Person
// Skill
// Shop
// Item
// God
// Job
// AnimSet
// Params
// Chapter
// AssetTable
// Animal
// Calculator

const SUPPORTED_GAMEDATAS: &[(&str, &str)] = &[
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
];

static GAMEDATA_MAP: phf::Map<&'static str, &'static str> = phf_map! {
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
};
fn main() {
    let cli = Args::parse();
    let mod_path = cli.mod_path;
    let romfs_path = mod_path.join("romfs");
    let mut target_path = mod_path.file_name().unwrap().to_str().unwrap().to_string();
    target_path.push_str(" (Cobalt)");
    println!("Mod name: {}", &target_path);
    let is_romfs: bool = romfs_path.is_dir();

    if !is_romfs {
        println!("This isn't a romfs folder, so I can't help you here.");
    }

    fs::create_dir_all(Path::new(&target_path).join("patches/xml"));
    fs::create_dir_all(Path::new(&target_path).join("patches/msbt"));

    fs::create_dir_all(Path::new(&target_path).join("Data"));

    for entry in WalkDir::new(Path::new(&mod_path).join("romfs"))
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let relative_path = diff_paths(path, &romfs_path).unwrap();
        println!("{}", relative_path.display());
        if path.is_dir() {
            fs::create_dir_all(Path::new(&target_path).join(relative_path));
            continue;
        }

        let file_name = entry.file_name().to_str().unwrap();
        // check if file is in SUPPORTED_GAMEDATAS
        // if it is, then we need to load it and do stuff
        // if it isn't, then we need to copy it over
        if file_name.ends_with(".xml.bundle") {
            let file_name = file_name.trim_end_matches(".xml.bundle");
            let supported_gamedata = GAMEDATA_MAP.contains_key(file_name);
            println!("Is {} supported: {}", file_name, supported_gamedata);
            if supported_gamedata {
                let new_name = GAMEDATA_MAP.get(file_name).unwrap();
                println!("Migrating {} to {}", file_name, new_name);
                migrate_gamedata(&path.to_path_buf(), new_name, &target_path);
            } else {
                println!("Copying {} to {}", file_name, &target_path);
                fs::copy(path, Path::new(&target_path).join(relative_path)).expect("died");
            }
        }

        if file_name.ends_with(".bytes.bundle") {
            let my_message = MessageBundle::load(path);
            match my_message {
                Ok(mut message) => {
                    println!("Message loaded successfully.");
                    message.serialize();
                    let my_result = message.serialize();
                    // println!("Bundle: {:?}", my_result);
                    let mut file = File::create(
                        Path::new(&target_path)
                            .join("patches")
                            .join("msbt")
                            .join(file_name)
                            .with_extension("msbt"),
                    )
                    .unwrap();
                    file.write_all(my_result.unwrap().as_slice()).expect("died");
                }
                Err(e) => {
                    println!("Error loading message: {:?}", e);
                }
            }
        }
    }

    // check_gamedata_folder(&mod_path);
}

fn migrate_gamedata(path: &PathBuf, new_name: &str, target_path: &str) {
    let my_bundle = TextBundle::load(path);

    match my_bundle {
        Ok(mut bundle) => {
            println!("Bundle loaded successfully.");
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
            file.write_all(my_result.as_slice()).expect("died");
        }
        Err(e) => {
            println!("Error loading bundle: {:?}", e);
        }
    }
}

fn check_gamedata_folder(mod_path: &PathBuf) -> bool {
    let gamedata_path = mod_path.join(GAMEDATA_PATH);
    let is_gamedata: bool = gamedata_path.is_dir();

    if !is_gamedata {
        println!("No gamedata to migrate.");
    } else {
        println!("Found gamedata to migrate.");
    }

    let (cobalt_supported, not_cobalt_supported): (Vec<_>, Vec<_>) = Path::new(&gamedata_path)
        .read_dir()
        .unwrap() // then just get DirEntry
        .map(|entry| entry.map(|e| e.path()).unwrap())
        .partition(|entry| {
            let binding = entry.file_name();
            SUPPORTED_GAMEDATAS.iter().any(|(name, _)| {
                let file_name: String = format!("{}.xml.bundle", name);
                &file_name == &binding.unwrap().to_str().unwrap()
            })
        });

    println!("Cobalt supported: {:?}", cobalt_supported);
    println!("Not cobalt supported: {:?}", not_cobalt_supported);

    cobalt_supported.into_iter().map(|path| {
        let my_bundle = TextBundle::load(path);

        match my_bundle {
            Ok(mut bundle) => {
                println!("Bundle loaded successfully.");
                let my_result = bundle.take_raw().unwrap();
                // println!("Bundle: {:?}", my_result);
                let mut file = File::create("foo.txt").unwrap();
                file.write_all(my_result.as_slice()).expect("died");
            }
            Err(e) => {
                println!("Error loading bundle: {:?}", e);
            }
        }
    });

    // let bundle = Bundle::load(Path::new(&gamedata_path).join("skill.xml.bundle"));

    // match bundle {
    //     Ok(bundle) => {
    //         println!("Bundle loaded successfully.");
    //         bundle.files().for_each(|(_, bundle_file)| {
    //             match bundle_file {
    //                 BundleFile::Assets(f) => {
    //                     f.assets.iter().for_each(|w| match w {
    //                         astra_formats::Asset::Text(t) => {
    //                             println!("Text: {:?}", t);
    //                         }
    //                         _ => {}
    //                     });
    //                     println!("yo!")
    //                 }

    //                 _ => {
    //                     println!("That's a negatory, it's raw.");
    //                 }
    //             }
    //             // let book = Book::load(&file);
    //             // match book {
    //             //     Ok(book) => {
    //             //         println!("Book loaded successfully.");
    //             //         println!("Book: {:?}", book);
    //             //     }
    //             //     Err(e) => {
    //             //         println!("Error loading book: {:?}", e);
    //             //     }
    //             // }
    //         });
    //     }
    //     Err(e) => {
    //         println!("Error loading bundle: {:?}", e);
    //     }
    // }

    is_gamedata
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    mod_path: PathBuf,
}
