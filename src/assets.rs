use speedy2d::image::ImageHandle;
use speedy2d::Graphics2D;

pub struct Assets {
    pub tileset: ImageHandle,
    pub watch: WatchAssets,
    pub player: PlayerAssets,
}

pub struct PlayerAssets {
    pub image: ImageHandle,
}

pub struct WatchAssets {
    pub image: ImageHandle,
}

impl Assets {
    pub fn load(graphics: &mut Graphics2D) -> Self {
        Self {
            tileset: load_image(graphics, include_bytes!("../assets/tileset.png")),
            watch: WatchAssets {
                image: load_image(graphics, include_bytes!("../assets/watch/image.png")),
            },
            player: PlayerAssets {
                image: load_image(graphics, include_bytes!("../assets/player/image.png")),
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
