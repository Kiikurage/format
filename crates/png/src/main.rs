use fltk::app::App;
use fltk::enums::ColorDepth;
use fltk::image::RgbImage;
use fltk::prelude::{ImageExt, WidgetBase, WidgetExt};
use fltk::window::Window;
use png::png::Png;

///
/// Show png image.
///
/// Mac OS requires to open a new window in main thread and `cargo test` is not suitable.
///
fn main() {
    let bmp = Png::open("./resources/sample_800x600.png").unwrap();

    let mut window = Window::new(0, 0, bmp.width as i32, bmp.height as i32, "image");
    window.draw(move |f| {
        RgbImage::new(
            bmp.data.as_ref(),
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
