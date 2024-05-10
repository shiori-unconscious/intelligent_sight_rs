use crate::unified_item::UnifiedItem;
use anyhow::Result;

#[derive(Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: UnifiedItem<u8>,
}

impl Image {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        Ok(Image {
            width,
            height,
            data: UnifiedItem::new((width * height * 3) as usize)?, // 3 channels
        })
    }
}

impl Default for Image {
    fn default() -> Self {
        match Image::new(640, 640) {
            Ok(image) => image,
            Err(err) => {
                panic!(
                    "Failed to create default Image, allocation failure: {}",
                    err
                );
            }
        }
    }
}

#[derive(Clone)]
pub struct Tensor {
    pub size: u32,
    pub data: UnifiedItem<f32>,
}

impl Tensor {
    pub fn new(size: u32) -> Result<Self> {
        Ok(Tensor {
            size,
            data: UnifiedItem::new(size as usize)?,
        })
    }
}