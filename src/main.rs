#![windows_subsystem = "windows"]

use std::collections::HashSet;
use std::fs;
use std::num::NonZeroU32;
use std::rc::Rc;

use clap::Parser;
use winit::event::{ElementState, Event, MouseButton, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use serde_yaml as yaml;

mod game;
mod draw;
mod filesystem;
use game::{Game, Coords, SaveFile};
use draw::Canvas;
use filesystem::{prefix_path, Slide};


const SAVE_FILE_PATH: &str = "save.yaml";

/// FerrousTale, a simple slide based interactive story game engine
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Generate an example slide YAML
    #[arg(short, long)]
    example: bool,

    /// Recursively search the story tree for issues
    #[arg(short, long)]
    check: bool,
}




fn main() {
    let args = Args::parse();

    if args.example {
        fs::write("example_slide.yaml", 
            serde_yaml::to_string(&Slide::example()).unwrap()
        ).unwrap();
    }

    if args.check {
        let mut dir_slides_found = HashSet::new();
        filesystem::recursive_check_dir(prefix_path(&SaveFile::default().location), &mut dir_slides_found);
        let mut yaml_slides_visited = HashSet::new();
        filesystem::recursive_check_yaml(SaveFile::default().location, &mut yaml_slides_visited);
        for slide in dir_slides_found.difference(&yaml_slides_visited) {
            eprintln!("unreachable slide {slide:?}")
        }
        return
    }



    let event_loop = EventLoop::new().unwrap();
    let window = Rc::new(
        WindowBuilder::new()
        .with_resizable(false)
        .with_title("FerrousTale")
        .build(&event_loop)
        .unwrap());
    let context = softbuffer::Context::new(window.clone()).unwrap();
    let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();



    let save_file: SaveFile = match &fs::read_to_string(SAVE_FILE_PATH) {
        Ok(val) => {
            match yaml::from_str(val) {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("could not deserialize save file: {e}");
                    SaveFile::default()
                }
            }
        }
        Err(e) => {
            eprintln!("could not read save file: {e}");
            SaveFile::default()
        }
    };

    let mut game: Game = save_file.try_into().expect("root initialization");


    let mut canvas = Canvas::build(&game);

    let mut mouse_pos = Coords {x: 0, y: 0};

    

    // ~~~~~~~~~~~~~~~~ ^^^^ ~~~~~~~~~~~~~~~~ INIT CODE ~~~~~~~~~~~~~~~~ ^^^^ ~~~~~~~~~~~~~~~~ 


    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent { window_id, event: WindowEvent::RedrawRequested } if window_id == window.id() => {
                let (width, height) = {
                    let size = window.inner_size();
                    (size.width, size.height)
                };
                surface
                    .resize(
                        NonZeroU32::new(width).unwrap(),
                        NonZeroU32::new(height).unwrap(),
                    )
                    .unwrap();
                

                // ~~~~~~~~~~~~~~~~ vvvv ~~~~~~~~~~~~~~~~ REDRAW CODE ~~~~~~~~~~~~~~~~ vvvv ~~~~~~~~~~~~~~~~ 

                let mut buffer = surface.buffer_mut().unwrap();
                


                //dbg!(&game.location);
                //dbg!(&game.keys);

                
                canvas.draw_to_buffer(&mut buffer, width, height);

                let canvas_size = canvas.size();
                _ = window.request_inner_size(canvas_size);

                buffer.present().unwrap();


                // ~~~~~~~~~~~~~~~~ ^^^^ ~~~~~~~~~~~~~~~~ REDRAW CODE ~~~~~~~~~~~~~~~~ ^^^^ ~~~~~~~~~~~~~~~~ 

            }
            Event::WindowEvent { 
                event: WindowEvent::MouseInput { 
                    device_id: _, 
                    state: ElementState::Pressed, 
                    button: MouseButton::Left
                },
                window_id,
            } if window_id == window.id() => {
                
                // ~~~~~~~~~~~~~~~~ vvvv ~~~~~~~~~~~~~~~~ INOUT CODE ~~~~~~~~~~~~~~~~ vvvv ~~~~~~~~~~~~~~~~ 

                if let Some((button_path, keys_added, keys_removed)) = canvas.click(mouse_pos.x, mouse_pos.y) {

                    if let Err(e) = game.goto(&button_path) {
                        eprintln!("could not go to slide {button_path:?}: {e}");
                    } else {
                        game.keys.extend(keys_added);
                        for key in keys_removed {
                            game.keys.remove(&key);
                        }

                        match serde_yaml::to_string(&SaveFile::from(&game)) {
                            Ok(yaml) => {
                                if let Err(e) = fs::write(SAVE_FILE_PATH, yaml) {
                                    eprintln!("could not write save file: {e}")
                                }
                            }
                            Err(e) => {eprintln!("could not serialize save file: {e}");}
                        }

                        canvas = Canvas::build(&game);
                        window.request_redraw();
                    }

                }

                // ~~~~~~~~~~~~~~~~ ^^^^ ~~~~~~~~~~~~~~~~ INPUT CODE ~~~~~~~~~~~~~~~~ ^^^^ ~~~~~~~~~~~~~~~~ 

            }
            Event::WindowEvent { 
                event: WindowEvent::CursorMoved { device_id: _, position },
                window_id,
            } if window_id == window.id() => {
                mouse_pos =  Coords {
                    x: position.x as i64,
                    y: position.y as i64,
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                window_id
            } if window_id == window.id() => {
                window.request_redraw();
            }
            
            _ => {}
        }
    }).unwrap();
}



