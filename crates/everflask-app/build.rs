//! Renders the SVG icons in `assets/icons` to PNG at build time, so the
//! shipped binary contains no SVG parser.

use std::{env, fs, path::Path};

/// 44px = 22pt at Retina, the menu-bar icon height cap.
const CANVAS: u32 = 44;
/// Artwork drawn at 36px, leaving breathing room like native menu-bar icons.
const ART: f32 = 36.0;

const ICONS: [&str; 10] = [
    "Light-Off",
    "Light-Off-Grey",
    "Light-On-R",
    "Light-On-G",
    "Light-On-B",
    "Dark-Off",
    "Dark-Off-Grey",
    "Dark-On-R",
    "Dark-On-G",
    "Dark-On-B",
];

fn main() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/icons");
    let out = Path::new(&env::var("OUT_DIR").unwrap()).to_path_buf();
    println!("cargo:rerun-if-changed={}", src.display());

    for name in ICONS {
        let path = src.join(format!("{name}.svg"));
        let data = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let tree = resvg::usvg::Tree::from_data(&data, &resvg::usvg::Options::default())
            .unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));

        let size = tree.size();
        let scale = ART / size.width().max(size.height());
        let offset = (CANVAS as f32 - ART) / 2.0;
        let transform =
            resvg::tiny_skia::Transform::from_scale(scale, scale).post_translate(offset, offset);

        let mut pixmap = resvg::tiny_skia::Pixmap::new(CANVAS, CANVAS).unwrap();
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        let png = pixmap.encode_png().expect("encode png");
        fs::write(out.join(format!("{name}.png")), png).unwrap();
    }
}
