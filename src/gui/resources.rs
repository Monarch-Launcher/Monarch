use iced::widget::image;
use once_cell::sync::Lazy;

pub static LOGO: Lazy<image::Handle> = Lazy::new(|| {
    let bytes = include_bytes!("../../icons/Square71x71Logo.png");
    image::Handle::from_bytes(bytes.to_vec())
});

pub static ICON: Lazy<image::Handle> = Lazy::new(|| {
    let bytes = include_bytes!("../../icons/icon.png");
    image::Handle::from_bytes(bytes.to_vec())
});
