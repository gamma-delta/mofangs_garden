#![allow(clippy::eval_order_dependence)]

use std::path::PathBuf;

use macroquad::prelude::*;
use once_cell::sync::Lazy;

pub struct Assets {
    pub textures: Textures,
    pub font: Font,
}

impl Assets {
    pub async fn init() -> Self {
        Self {
            textures: Textures::init().await,
            font: font("source_serif").await,
        }
    }
}

pub struct Textures {
    pub wood: Texture2D,
    pub fire: Texture2D,
    pub earth: Texture2D,
    pub metal: Texture2D,
    pub water: Texture2D,
    pub heavenly: Texture2D,
    pub earthly: Texture2D,
    pub human: Texture2D,
    pub yin: Texture2D,
    pub yang: Texture2D,
    pub change: Texture2D,
    pub qi: Texture2D,

    pub highlight: Texture2D,
    pub select: Texture2D,
}

impl Textures {
    async fn init() -> Self {
        Self {
            wood: texture("wood").await,
            fire: texture("fire").await,
            earth: texture("earth").await,
            metal: texture("metal").await,
            water: texture("water").await,
            heavenly: texture("heavenly").await,
            earthly: texture("earthly").await,
            human: texture("human").await,
            yin: texture("yin").await,
            yang: texture("yang").await,
            change: texture("change").await,
            qi: texture("qi").await,

            highlight: texture("highlight").await,
            select: texture("select").await,
        }
    }
}

/// Path to the assets root
static ASSETS_ROOT: Lazy<PathBuf> = Lazy::new(|| {
    if cfg!(debug_assertions) {
        if cfg!(target_arch = "wasm32") {
            PathBuf::from("../assets")
        } else {
            PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets"))
        }
    } else {
        todo!("assets path for release hasn't been finalized yet ;-;")
    }
});

async fn texture(path: &str) -> Texture2D {
    let with_extension = path.to_owned() + ".png";
    load_texture(
        ASSETS_ROOT
            .join("textures")
            .join(with_extension)
            .to_string_lossy()
            .as_ref(),
    )
    .await
}

async fn font(path: &str) -> Font {
    let with_extension = path.to_owned() + ".ttf";
    load_ttf_font(ASSETS_ROOT.join(with_extension).to_string_lossy().as_ref()).await
}
