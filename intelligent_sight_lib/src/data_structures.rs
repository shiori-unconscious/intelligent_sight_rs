use crate::unified_item::UnifiedItem;
use anyhow::Result;
use std::ops::{Deref, DerefMut};
use std::time::Instant;

pub struct ImageBuffer {
    pub width: u32,
    pub height: u32,
    data: UnifiedItem<u8>,
    pub timestamp: Instant,
}

impl ImageBuffer {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        Ok(ImageBuffer {
            width,
            height,
            data: UnifiedItem::new((width * height * 3) as usize)?, // 3 channels
            timestamp: Instant::now(),
        })
    }
}

impl Default for ImageBuffer {
    fn default() -> Self {
        match ImageBuffer::new(640, 640) {
            Ok(image) => image,
            Err(err) => {
                panic!(
                    "Failed to create default ImageBuffer, allocation failure: {}",
                    err
                );
            }
        }
    }
}

impl Deref for ImageBuffer {
    type Target = UnifiedItem<u8>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for ImageBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl Clone for ImageBuffer {
    fn clone(&self) -> Self {
        let mut data = UnifiedItem::new(self.data.len()).expect("fail to malloc UnifiedItem<u8>");
        data.iter_mut()
            .zip(self.data.iter())
            .for_each(|(dst, src)| *dst = *src);
        ImageBuffer {
            width: self.width,
            height: self.height,
            data,
            timestamp: self.timestamp,
        }
    }
}

pub struct TensorBuffer {
    size: Vec<usize>,
    data: UnifiedItem<f32>,
    timestamp: Instant,
}

impl TensorBuffer {
    pub fn new(size: Vec<usize>) -> Result<Self> {
        Ok(TensorBuffer {
            data: UnifiedItem::new(size.iter().fold(1, |sum, num| sum * num))?,
            size,
            timestamp: Instant::now(),
        })
    }

    pub fn size(&self) -> &Vec<usize> {
        &self.size
    }

    pub fn resize(&mut self, size: Vec<usize>) {
        self.size = size;
    }
}

impl Deref for TensorBuffer {
    type Target = UnifiedItem<f32>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for TensorBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl Clone for TensorBuffer {
    fn clone(&self) -> Self {
        let mut data = UnifiedItem::new(self.data.len()).expect("fail to malloc UnifiedItem<f32>");
        data.iter_mut()
            .zip(self.data.iter())
            .for_each(|(dst, src)| *dst = *src);
        TensorBuffer {
            size: self.size.clone(),
            data,
            timestamp: self.timestamp,
        }
    }
}
