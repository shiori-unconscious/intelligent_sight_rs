mod image;

use anyhow::{Ok, Result};
pub use image::Image;

const ERR_NAME: [&'static str; 62] = ["操作成功", "操作失败", "内部错误", "未知错误", "不支持该功能", "初始化未完成", "参数无效", "参数越界", "未使能", "用户手动取消了，比如roi面板点击取消，返回", "注册表中没有找到对应的路径", "获得图像数据长度和定义的尺寸不匹配", "超时错误", "硬件IO错误", "通讯错误", "总线错误", "没有发现设备", "未找到逻辑设备", "设备已经打开", "设备已经关闭", "没有打开设备视频，调用录像相关的函数时，如果相机视频没有打开，则回返回该错误。", 
"没有足够系统内存", "创建文件失败", "文件格式无效", "写保护，不可写", "数据采集失败", "数据丢失，不完整", "未接收到帧结束符", "正忙(上一次操作还在进行中)，此次操作不能进行", "需要等待(进行操作的条件不成立)，可以再次尝试trf", "正在进行，已经被操作过", "IIC传输错误", "SPI传输错误", "USB控制传输错误", "USBBULK传输错误", "网络传输套件初始化失败", "网络相机内核过滤驱动初始化失败，请检查是否正确安装了驱动，或者重新安装。", "网络数据发送错误", "与网络相机失去连接，心跳检测超时", 
"接收到的字节数比请求的少", "从文件中加载程序失败", "程序运行所必须的文件丢失。", "固件和程序不匹配，原因是下载了错误的固件。", "参数超出有效范围。", "安装程序注册错误。请重新安装程序，或者运行安装目录Setup/Installer.exe", "禁止访问。指定相机已经被其他程序占用时，再申请访问该相机，会返回该状态。(一个相机不能被多个程序同时访问)", "表示相机需要复位后才能正常使用，此时请让相机断电重启，或者重启操作系统后，便可正常使用。", "ISP模块未初始化", "数据校验错误", "数据测试失败", "内部错误1", 
"U3V控制端点未找到", "U3V控制通讯错误", r#"无效的设备名，名字里不能包含以下字符(\/:*?"<>|")"#, "格式错误", "PCIE设备打开失败", "PCIE设备通讯失败", "PCIEDDR错误",
"指定相机不存在", "找到的相机数量少于指定值", "相机输出图片格式错误", "相机输出图片大小错误"];

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
                ERR_NAME.get(err_code as usize).expect("Invalid error code")
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
            ERR_NAME.get(err_code as usize).expect("Invalid error code")
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
            ERR_NAME.get(err_code as usize).expect("Invalid error code")
        ))),
    }
}
