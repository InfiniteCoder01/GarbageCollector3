use speedy2d::image::ImageHandle;
use speedy2d::Graphics2D;

pub struct Assets {
    pub font: speedy2d::font::Font,
    pub tileset: ImageHandle,
    pub particles: ImageHandle,

    pub watch: WatchAssets,
    pub player: PlayerAssets,
}

pub struct PlayerAssets {
    pub image: ImageHandle,
    pub image_flip: ImageHandle,
    pub image_nowatch: ImageHandle,
}

pub struct WatchAssets {
    pub image: ImageHandle,
    pub icons: ImageHandle,
}

impl Assets {
    pub fn load(graphics: &mut Graphics2D) -> Self {
        Self {
            font: speedy2d::font::Font::new(include_bytes!("../assets/712_serif.ttf")).unwrap(),
            tileset: load_image(graphics, include_bytes!("../assets/tileset.png")),
            particles: load_image(graphics, include_bytes!("../assets/particles.png")),

            watch: WatchAssets {
                image: load_image(graphics, include_bytes!("../assets/watch/image.png")),
                icons: load_image(graphics, include_bytes!("../assets/watch/icons.png")),
            },
            player: PlayerAssets {
                image: load_image(graphics, include_bytes!("../assets/player/image.png")),
                image_flip: load_image(graphics, include_bytes!("../assets/player/image_flip.png")),
                image_nowatch: load_image(graphics, include_bytes!("../assets/player/image_nowatch.png")),
            },
        }
    }
}

fn load_image(graphics: &mut Graphics2D, bytes: &[u8]) -> ImageHandle {
    graphics
        .create_image_from_file_bytes(
            None,
            speedy2d::image::ImageSmoothingMode::NearestNeighbor,
            std::io::Cursor::new(bytes),
        )
        .unwrap()
}
