use bevy::prelude::Image;

pub trait ImageExt {
    fn dimensions(&self) -> (u32, u32);
    fn get_pixel(&self, x: u32, y: u32) -> [u8; 3];
}

impl ImageExt for Image {
    fn dimensions(&self) -> (u32, u32) {
        let size = self.size();
        (size.x, size.y)
    }

    fn get_pixel(&self, x: u32, y: u32) -> [u8; 3] {
        let size = self.size();
        let data = &self.data;
        let index = (y * size.x + x) as usize * 4;
        [data[index], data[index + 1], data[index + 2]]
    }
}