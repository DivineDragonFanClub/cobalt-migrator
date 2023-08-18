use astra_formats::{Book, TextBundle};
use astra_formats::{Bundle, BundleFile};
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;

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

fn main() {
    let cli = Args::parse();
    let mod_path = cli.mod_path;

    let is_romfs: bool = Path::new(&mod_path).join("romfs").is_dir();

    if !is_romfs {
        println!("This isn't a romfs folder, so I can't help you here.");
    }

    check_gamedata_folder(&mod_path);
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
                let file_name = format!("{}.xml.bundle", name);
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
