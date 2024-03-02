use std::{collections::HashSet, path::PathBuf, rc::Rc};
use image::{imageops::overlay, DynamicImage, Pixel, Rgb};
use softbuffer::Buffer;
use winit::{dpi::PhysicalSize, window::Window};
use crate::{filesystem::{read_image, FallbackAsset}, game::{Coords, Game}};


// Written by soweli Luna

#[derive(Default)]
pub struct Canvas {
    dynamic_image: DynamicImage,
    pub buttons: Vec<Button>,
    pub size: Coords<i32>,
}
impl Canvas {
    pub fn draw_to_buffer(&self, buffer: &mut Buffer<'_, Rc<Window>, Rc<Window>>, width: u32, height: u32) {
        let image = self.dynamic_image.clone().into_rgb8();
        for index in 0..(width * height) {
            let y = index / width;
            let x = index % width;

            let pixel = 
                image
                .get_pixel_checked(x, y)
                .unwrap_or(&Rgb::<u8>::from([0, 0, 0]))
                .to_rgb();

            let pixel_channels = pixel.channels();
            
            let u32_pixel: u32 = 
                (pixel_channels[0] as u32) << 16 |
                (pixel_channels[1] as u32) << 8  |
                (pixel_channels[2] as u32);

            buffer[index as usize] = u32_pixel;
            
        }
    }

    pub fn build(game: &Game) -> Self {
        let mut canvas = Canvas::default();

        canvas.dynamic_image = read_image(
            game.location.join(game.slide.background_path.clone()), 
            FallbackAsset::Background
        );
        canvas.size = Coords {
            x: canvas.dynamic_image.width() as i32,
            y: canvas.dynamic_image.height() as i32,
        };

        for element in &game.slide.nonclickables {
            let image = &read_image(
                game.location.join(element.image_path.clone()), 
                FallbackAsset::Nonclickable
            );
            let Coords {x, y} = canvas.position_asset(
                element.position, 
                element.anchor, 
                element.offset, 
                Coords { 
                    x: image.width() as i32, 
                    y: image.height() as i32,
                }
            );
            overlay(
                &mut canvas.dynamic_image, 
                image, 
                x as i64, 
                y as i64, 
            )
        } 

        for element in &game.slide.clickables {
            if element.must_have_keys.is_subset(&game.keys) {
                let image = &read_image(
                    game.location.join(element.image_path.clone()), 
                    FallbackAsset::Clickable
                );
                let Coords {x, y} = canvas.position_asset(
                    element.position, 
                    element.anchor, 
                    element.offset, 
                    Coords { 
                        x: image.width() as i32, 
                        y: image.height() as i32,
                    }
                );
                canvas.buttons.push(Button { 
                    slide_path: element.slide_path.clone(), 
                    adds_keys: element.adds_keys.clone(),
                    removes_keys: element.removes_keys.clone(),
                    x1: x as i64, 
                    y1: y as i64, 
                    x2: x as i64 + image.width() as i64, 
                    y2: y as i64 + image.height() as i64, 
                });
                overlay(
                    &mut canvas.dynamic_image, 
                    image, 
                    x as i64, 
                    y as i64,
                )
            }
            
        } 

        canvas
    }

    pub fn click(&self, x: i64, y: i64) -> Option<(PathBuf, HashSet<String>, HashSet<String>)> {
        for button in &self.buttons {
            if (button.x1..button.x2).contains(&x) && (button.y1..button.y2).contains(&y) {
                return Some((
                    button.slide_path.clone(), 
                    button.adds_keys.clone(), 
                    button.removes_keys.clone()
                ))
            }
        };
        None
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        PhysicalSize { 
            width: self.dynamic_image.width(), 
            height: self.dynamic_image.height() 
        }
    }

    fn position_asset(
        &self, 
        position: Coords<f32>, 
        anchor: Coords<f32>, 
        offset: Coords<i32>, 
        element_size: Coords<i32>
    ) -> Coords<i32> {
        let position_offset = (position * self.size.map(|t|{t as f32})).map(|t|{t as i32});
        let anchor_offset = (anchor * element_size.map(|t|{t as f32})).map(|t|{t as i32});
        position_offset + offset - anchor_offset
    }
}


pub struct Button {
    pub slide_path: PathBuf,
    pub adds_keys: HashSet<String>,
    pub removes_keys: HashSet<String>,
    pub x1: i64,
    pub y1: i64,
    pub x2: i64,
    pub y2: i64,
}

