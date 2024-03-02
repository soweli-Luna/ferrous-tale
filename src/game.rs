use std::{collections::HashSet, fs, ops::{Add, Mul, Sub}, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::filesystem::{canonical_join, prefix_path, Slide, ASSETS};



// Written by soweli Luna

#[derive(Debug)]
pub struct Game {
    pub location: PathBuf,
    pub keys: HashSet<String>,
    pub slide: Slide,
}
impl Game {
    pub fn goto(&mut self, path: &PathBuf) -> Result<(), String> {
        let new_location = self.cd(path)?;
        self.slide = Slide::read_yaml(&new_location)?;
        self.location = new_location;
        Ok(())
    }
    fn cd(& self, path: &PathBuf) -> Result<PathBuf, String>{
        let try_path = canonical_join(&self.location, path);
        if cfg!(feature="portable") {
            match fs::read_dir(&prefix_path(&try_path)) {
                Ok(_) => Ok(try_path),
                Err(e) => Err(format!("could not find directory {try_path:?}: {e}")),
            }
        } else {    //for static assets
            let entry = ASSETS.get_entry(&prefix_path(&try_path))
                .ok_or(format!("could not find static directory {try_path:?}"))?;
            if let include_dir::DirEntry::Dir(_) = entry {
                return Ok(try_path)
            }
            Err(format!("static {path:?} is not a directory"))
        }
    }
}
impl TryFrom<SaveFile> for Game {
    type Error = String;

    fn try_from(value: SaveFile) -> Result<Self, Self::Error> {
        Ok(Game {
            location: value.location.clone(),
            keys: value.keys,
            slide: Slide::read_yaml(&value.location)?,
        })
    }
}

#[derive(Deserialize, Serialize)]
pub struct SaveFile {
    pub location: PathBuf,
    keys: HashSet<String>,
}
impl From<&Game> for SaveFile {
    fn from(game: &Game) -> Self {
        Self { 
            location: game.location.clone(), 
            keys: game.keys.clone(),
        }
    }
    
}
impl Default for SaveFile {
    fn default() -> Self {
        Self { 
            location: "/".into(), 
            keys: Default::default(),
        }
    }
}



#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Nonclickable {
    pub image_path: PathBuf,
    #[serde(default)]
    pub position: Coords<f32>,
    #[serde(default)]
    pub anchor: Coords<f32>,
    #[serde(default)]
    pub offset: Coords<i32>,
    //delay: f32,
}


#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Clickable {
    pub image_path: PathBuf,
    pub slide_path: PathBuf,
    #[serde(default)]
    pub position: Coords<f32>,
    #[serde(default)]
    pub anchor: Coords<f32>,
    #[serde(default)]
    pub offset: Coords<i32>,
    #[serde(default)]
    pub adds_keys: HashSet<String>,
    #[serde(default)]
    pub removes_keys: HashSet<String>,
    #[serde(default)]
    pub must_have_keys: HashSet<String>,
    #[serde(default)]
    pub mustnt_have_keys: HashSet<String>,
}



#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct Coords<T> {
    pub x: T,
    pub y: T,
}
impl<T: Add<Output = T>> Add for Coords<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl<T: Sub<Output = T>> Sub for Coords<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl<T: Mul<Output = T>> Mul for Coords<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}
impl<T: Copy> Coords<T> {
    pub fn map<U, F>(&self, mut f: F) -> Coords<U>
    where 
        F: FnMut(T) -> U
    {
        Coords::<U> {
            x: f(self.x),
            y: f(self.y),
        }
    }
}



