use parking_lot::Mutex;
use std::sync::Arc;

use crate::canvas::Canvas;
use crate::image::SharedImage;
use crate::rectarea::RectArea;
use crate::types::Tile;
use crate::utility::{parse_hex_string, simplify_string};

pub struct Tilemap {
    width: u32,
    height: u32,
    self_rect: RectArea,
    clip_rect: RectArea,
    data: Vec<Vec<Tile>>,
    pub image: SharedImage,
}

pub type SharedTilemap = Arc<Mutex<Tilemap>>;

impl Tilemap {
    pub fn new(width: u32, height: u32, image: SharedImage) -> SharedTilemap {
        Arc::new(Mutex::new(Tilemap {
            width: width,
            height: height,
            self_rect: RectArea::new(0, 0, width, height),
            clip_rect: RectArea::new(0, 0, width, height),
            data: vec![vec![(0, 0); width as usize]; height as usize],
            image: image,
        }))
    }

    pub fn set(&mut self, x: i32, y: i32, data_str: &[&str]) {
        let width = data_str[0].len() as u32 / 4;
        let height = data_str.len() as u32;
        let tilemap = Tilemap::new(width, height, self.image.clone());

        {
            let mut tilemap = tilemap.lock();

            for i in 0..height {
                let src_data = simplify_string(data_str[i as usize]);

                for j in 0..width {
                    let index = j as usize * 4;

                    if let Some(value) = parse_hex_string(&src_data[index..index + 4]) {
                        tilemap._set_value(
                            j as i32,
                            i as i32,
                            (((value >> 16) & 0xff) as u8, (value & 0xff) as u8),
                        );
                    } else {
                        panic!("invalid tilemap data");
                    }
                }
            }
        }

        self.blt(x, y, tilemap, 0, 0, width as i32, height as i32, None, None);
    }
}

impl Canvas<Tile> for Tilemap {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn _value(&self, x: i32, y: i32) -> Tile {
        self.data[y as usize][x as usize]
    }

    fn _set_value(&mut self, x: i32, y: i32, tile: Tile) {
        self.data[y as usize][x as usize] = tile;
    }

    fn _self_rect(&self) -> RectArea {
        self.self_rect
    }

    fn _clip_rect(&self) -> RectArea {
        self.clip_rect
    }

    fn _clip_rect_mut(&mut self) -> &mut RectArea {
        &mut self.clip_rect
    }
}
