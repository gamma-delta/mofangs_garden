#![allow(clippy::eval_order_dependence)]

use std::path::PathBuf;

use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::*,
};
use once_cell::sync::Lazy;

pub struct Assets {
    pub textures: Textures,
    pub font: Font,
    pub fade_shader: Material,
}

impl Assets {
    pub async fn init() -> Self {
        Self {
            textures: Textures::init().await,
            font: font("source_serif").await,
            fade_shader: fade_shader(),
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

    pub hex: Texture2D,
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
            hex: texture("hex").await,
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

fn fade_shader() -> Material {
    load_material(
        r#"
#version 100
precision lowp float;

attribute vec3 position;
attribute vec4 color0;
attribute vec2 texcoord;

varying vec2 uv;
varying vec2 pos;
varying vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
    color = color0 / 255.0;
}
    "#,
        r#"
#version 100
precision lowp float;

varying vec2 uv;
varying vec4 color;

uniform sampler2D Texture;

vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void main() {
    vec4 baseColor = texture2D(Texture, uv) * color;
    vec3 hsv = rgb2hsv(baseColor.rgb);
    hsv.z = hsv.z * 0.5 + 0.4;
    hsv.y *= 0.2; // desaturate
    gl_FragColor = vec4(hsv2rgb(clamp(hsv, vec3(0.0), vec3(1.0))), baseColor.a);
}

    "#,
        // Make alpha work
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap()
}
