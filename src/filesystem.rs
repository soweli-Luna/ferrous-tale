use std::{collections::HashSet, fs, path::{Component, Path, PathBuf}};

use image::DynamicImage;
use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};
use serde_yaml as yaml;

use crate::game::{Clickable, Coords, Nonclickable};


pub static ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets");

const ROOT_PREFIX: &str = "story/";
const YAML_FILENAME: &str = "slide.yaml";


// Written by soweli Luna

pub fn read_image(path: PathBuf, fallback: FallbackAsset) -> DynamicImage {
    let full_path = prefix_path(&path);

    if cfg!(feature="portable") {

        match image::open(&full_path) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("could not find asset {full_path:?}: {e}");
                image::load_from_memory(
                    ASSETS.get_file(fallback.clone().into_pathbuf())
                    .expect("could not find static fallback asset")
                    .contents()
                ).expect("could not load static fallback asset")
            }
        }

    } else {    //for static assets

        match image::load_from_memory(
            ASSETS.get_file(&full_path)
            .unwrap_or_else(|| {
                eprintln!("could not find static asset {full_path:?}");
                ASSETS.get_file(fallback.clone().into_pathbuf())
                .expect("could not find static fallback asset")
            }).contents()
        ) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("could not decode static asset {full_path:?}: {e}");
                image::load_from_memory(
                    ASSETS.get_file(fallback.into_pathbuf())
                    .expect("could not find static fallback asset")
                    .contents()
                ).expect("could not decode static fallback asset")
            }
        }

    }
}
#[derive(Clone)]
pub enum FallbackAsset {
    Background,
    Clickable,
    Nonclickable,
}
impl FallbackAsset {
    fn into_pathbuf(self) -> PathBuf {
        match self {
            Self::Background => "fallback/bg.bmp".into(),
            Self::Clickable => "fallback/clickable.bmp".into(),
            Self::Nonclickable =>  "fallback/clickable.bmp".into(),
        }
    }
}


pub fn prefix_path(path: &PathBuf) -> PathBuf {
    let root_prefix: PathBuf = ROOT_PREFIX.into(); 
    root_prefix.join(
        path
        .strip_prefix("/")
        .unwrap_or(
            path
            .as_path()
        )
    )
}

pub fn canonical_join(lhs: &PathBuf, rhs: &PathBuf) -> PathBuf {
    let rhs_first_component= rhs.components().nth(0);
    match rhs_first_component {
        Some(Component::CurDir) => {
            lhs.join(rhs.strip_prefix("./").unwrap_or(
                rhs.strip_prefix(".").expect("canonical join prefix strip")
            ))
        },
        Some(Component::ParentDir) => {
            lhs.parent().unwrap_or(Path::new("")).join(rhs.strip_prefix("../").unwrap_or(
                rhs.strip_prefix("..").expect("canonical join prefix strip")
            ))
        },
        _ => {lhs.join(rhs)}
    }
}





#[derive(Serialize, Deserialize, Debug)]
pub struct Slide {
    pub background_path: PathBuf, 
    #[serde(default)]
    pub nonclickables: Vec<Nonclickable>, 
    pub clickables: Vec<Clickable>,
}
impl Slide {
    pub fn read_yaml(input: &PathBuf) -> Result<Self, String> {
        let full_path = prefix_path(input).join(YAML_FILENAME);
        if cfg!(feature="portable") {
            match yaml::from_str(
                &match fs::read_to_string(&full_path) {
                    Ok(val) => val,
                    Err(e) => {Err(format!("could not find {full_path:?}: {e}"))?},
                }
            ) {
                Ok(val) => Ok(val),
                Err(e) => Err(format!("could not read {full_path:?}: {e}")),
            }
        } else {    //for static assets
            match yaml::from_slice(
                &ASSETS.get_file(&full_path).ok_or(&format!("could not find static {full_path:?}"))?.contents()
            ) {
                Ok(val) => Ok(val),
                Err(e) => Err(format!("could not read static {full_path:?}: {e}")),
            }
        }
        
    }

    pub fn example() -> Self {
        let example_keyset = HashSet::from(["key1".into(), "key2".into()]);
        Self {
            background_path: "path".into(),
            nonclickables: vec![Nonclickable {
                image_path: "path2".into(),
                position: Coords {x: 0.0, y: 0.0},
                anchor: Coords {x: 0.0, y: 0.0},
                offset: Coords {x: 0, y: 0},
            }],
            clickables: vec![Clickable { 
                image_path: "path3".into(), 
                position: Coords {x: 0.0, y: 0.0}, 
                anchor: Coords {x: 0.0, y: 0.0}, 
                offset: Coords {x: 0, y: 0}, 
                slide_path: "path4".into(), 
                adds_keys: example_keyset.clone(), 
                removes_keys: example_keyset.clone(), 
                must_have_keys: example_keyset.clone(), 
                mustnt_have_keys: example_keyset, 
            }],
        }
    }
}




pub fn recursive_check_dir(path: PathBuf, slides_found: &mut HashSet<PathBuf>) {
    if cfg!(feature="portable") {

        for entry in fs::read_dir(&path).expect(&format!("recursive check {path:?}")) {
            match entry {
                Ok(val) => {
                    if val.file_name() == YAML_FILENAME {
                        slides_found.insert(val.path());
                    }
                    if val.path().is_dir() { 
                        recursive_check_dir(val.path(), slides_found);
                    } 
                },
                Err(e) => {eprintln!("could not read directory entry {path:?}: {e}");},
            }
        }

    } else {    // for static assets
        if ASSETS.contains(path.join(YAML_FILENAME)) {
            slides_found.insert(path.join(YAML_FILENAME));
        }
        for dir in ASSETS.get_dir(&path).unwrap().dirs() {
            recursive_check_dir(dir.path().into(), slides_found);
            
        }
    }
}


pub fn recursive_check_yaml(path: PathBuf, slides_visited: &mut HashSet<PathBuf>) {
    
    match Slide::read_yaml(&path) {
        Ok(slide) => {
            if !slides_visited.contains(&prefix_path(&path).join(YAML_FILENAME)) {
                slides_visited.insert(prefix_path(&path).join(YAML_FILENAME));
                read_image(canonical_join(&path, &slide.background_path), FallbackAsset::Background);

                for nonclickable in slide.nonclickables {
                    read_image(canonical_join(&path, &nonclickable.image_path), FallbackAsset::Nonclickable);
                }

                for clickable in slide.clickables {
                    recursive_check_yaml(canonical_join(&path, &clickable.slide_path), slides_visited);
                    read_image(canonical_join(&path, &clickable.image_path), FallbackAsset::Clickable);
                }
            }
        }
        Err(e) => {eprintln!("{e}")}
    };

}