use std::{env, fs, path::Path};

use resvg::{
    tiny_skia::{Pixmap, Transform},
    usvg::{Tree, TreeParsing},
};

#[cfg(windows)]
use winres::WindowsResource;

fn main() {
    println!("cargo:rerun-if-changed=./assets/icon.svg");

    let image = fs::read_to_string("./assets/icon.svg").expect("file should exist");
    let svg_tree = Tree::from_str(&image, &Default::default()).expect("image should be an svg");

    let mut pixmap = Pixmap::new(512, 512).expect("pixmap should be created");

    resvg::render(
        &svg_tree,
        resvg::FitTo::Size(512, 512),
        Transform::identity(),
        pixmap.as_mut(),
    )
    .expect("svg render should succeed");

    let out_dir = env::var("OUT_DIR").expect("env var OUT_DIR should exist");
    let mut png_path = Path::new(&out_dir).to_owned();

    png_path.push("icon.png");

    pixmap
        .save_png(&png_path)
        .expect("should be able to save png");

    #[cfg(windows)]
    {
        let mut ico_path = Path::new(&out_dir).to_owned();
        ico_path.push("icon.ico");

        let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
        let file = fs::File::open(png_path).expect("png should exist by this point");
        let image = ico::IconImage::read_png(file).expect("should be able to read png image");
        icon_dir.add_entry(
            ico::IconDirEntry::encode(&image).expect("should be able to encode created png"),
        );

        let file = fs::File::create(&ico_path).expect("should be able to create icon file");
        icon_dir
            .write(file)
            .expect("should be able to write icon dir to icon file");

        WindowsResource::new()
            .set_icon(
                ico_path
                    .as_os_str()
                    .to_str()
                    .expect("path should be a valid string"),
            )
            .compile()
            .expect("resource compilation should succeed");
    }
}
