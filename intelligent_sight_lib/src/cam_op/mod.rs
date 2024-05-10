use crate::Image;
use anyhow::Result;

mod err_name;
use err_name::MV_ERR_NAME;

pub enum FlipFlag {
    None,
    Vertical,
    Horizontal,
    Both,
}

mod cam_op_ffi {
    extern "C" {
        pub fn initialize_camera(
            wanted_cam_number: u8,
            image_width: *mut u32,
            image_height: *mut u32,
            already_initialized: *mut u8,
        ) -> u8;
        pub fn get_image(
            camera_index: u8,
            image_data: *mut u8,
            image_width: *mut u32,
            image_height: *mut u32,
            flip_flag: u8,
        ) -> u8;
        pub fn uninitialize_camera() -> u8;
    }
}

pub fn initialize_camera(
    wanted_cam_number: u8,
    buffer_width: &mut Vec<u32>,
    buffer_height: &mut Vec<u32>,
) -> Result<()> {
    let mut already_initialized: u8 = 0;
    match unsafe {
        cam_op_ffi::initialize_camera(
            wanted_cam_number,
            buffer_width.as_mut_ptr(),
            buffer_height.as_mut_ptr(),
            &mut already_initialized as *mut u8,
        )
    } {
        0 => Ok(()),
        err_code => {
            if already_initialized != 0 {
                let _ = uninitialize_camera();
            }
            Err(anyhow::anyhow!(format!(
                "Failed to initialize camera, err code: {} ({})",
                err_code,
                MV_ERR_NAME
                    .get(err_code as usize)
                    .unwrap_or(&"err code unknown")
            )))
        }
    }
}

pub fn get_image(camera_index: u8, image: &mut Image, flip_flag: FlipFlag) -> Result<()> {
    match unsafe {
        cam_op_ffi::get_image(
            camera_index,
            image.data.as_mut_ptr(),
            &mut image.width as *mut u32,
            &mut image.height as *mut u32,
            flip_flag as u8,
        )
    } {
        0 => Ok(()),
        err_code => Err(anyhow::anyhow!(format!(
            "Failed to get image, err code: {} ({})",
            err_code,
            MV_ERR_NAME
                .get(err_code as usize)
                .unwrap_or(&"err code unknown")
        ))),
    }
}

pub fn uninitialize_camera() -> Result<()> {
    println!("uninitialize");
    match unsafe { cam_op_ffi::uninitialize_camera() } {
        0 => Ok(()),
        err_code => Err(anyhow::anyhow!(format!(
            "Failed to uninitialize camera, err code: {} ({})",
            err_code,
            MV_ERR_NAME
                .get(err_code as usize)
                .unwrap_or(&"err code unknown")
        ))),
    }
}