use fltk::app::App;
use fltk::enums::ColorDepth;
use fltk::image::RgbImage;
use fltk::prelude::{ImageExt, WidgetBase, WidgetExt};
use fltk::window::Window;

use bmp::bmp::Bmp;

///
/// Show bmp image.
///
/// Mac OS requires to open a new window in main thread and `cargo test` is not suitable.
///
fn main() {
    let bmp = Bmp::open("./resources/sample_640x426.bmp").unwrap();

    let mut window = Window::new(0, 0, bmp.width as i32, bmp.height as i32, "image");
    window.draw(move |f| {
        RgbImage::new(
            bmp.as_normalized_rgb().as_ref(),
            bmp.width as i32,
            bmp.height as i32,
            ColorDepth::Rgb8,
        )
        .unwrap()
        .draw(0, 0, f.w(), f.h());
    });
    window.show();
    App::default().run().unwrap();
}
