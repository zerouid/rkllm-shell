//! Vision encoder module for multimodal support
//!
//! This module handles image preprocessing and embedding generation
//! for multimodal models (e.g., LLaVA, Qwen-VL).

use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use image::DynamicImage;
use std::path::Path;

// ---------------------------------------------------------------------------
// Image Processing Utilities
// ---------------------------------------------------------------------------

/// Supported image formats for multimodal input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisionVisionImageFormat {
    Png,
    Jpeg,
    WebP,
    Bmp,
}

impl VisionVisionImageFormat {
    pub fn from_mime(mime: &str) -> Option<Self> {
        match mime {
            "image/png" => Some(Self::Png),
            "image/jpeg" | "image/jpg" => Some(Self::Jpeg),
            "image/webp" => Some(Self::WebP),
            "image/bmp" => Some(Self::Bmp),
            _ => None,
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "webp" => Some(Self::WebP),
            "bmp" => Some(Self::Bmp),
            _ => None,
        }
    }
}

/// Decode a base64-encoded image string
/// 
/// Supports data URLs (e.g., "data:image/png;base64,...") and raw base64.
pub fn decode_base64_image(data: &str) -> Result<(DynamicImage, VisionVisionImageFormat)> {
    let base64_data = if data.starts_with("data:") {
        data.split(',').nth(1).ok_or_else(|| anyhow::anyhow!("Invalid data URL format"))?
    } else {
        data
    };

    let bytes = general_purpose::STANDARD
        .decode(base64_data)
        .context("Failed to decode base64 image")?;

    let format = detect_image_format(&bytes)
        .context("Unknown image format")?;

    let img = image::load_from_memory(&bytes)
        .context("Failed to load image from memory")?;

    Ok((img, format))
}

fn detect_image_format(bytes: &[u8]) -> Option<VisionVisionImageFormat> {
    if bytes.len() < 12 {
        return None;
    }
    if bytes[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        return Some(VisionVisionImageFormat::Png);
    }
    if bytes[0..3] == [0xFF, 0xD8, 0xFF] {
        return Some(VisionVisionImageFormat::Jpeg);
    }
    if bytes[0..4] == [0x52, 0x49, 0x46, 0x46] && bytes[8..12] == [0x57, 0x45, 0x42, 0x50] {
        return Some(VisionVisionImageFormat::WebP);
    }
    if bytes[0..2] == [0x42, 0x4D] {
        return Some(VisionVisionImageFormat::Bmp);
    }
    None
}

pub fn preprocess_image(
    img: DynamicImage,
    target_size: (u32, u32),
) -> Result<Vec<f32>> {
    let resized = img.resize_exact(
        target_size.0,
        target_size.1,
        image::imageops::FilterType::Triangle,
    );

    let rgb = resized.to_rgb8();

    let mean = [0.48145466_f32, 0.4578275, 0.40821073];
    let std = [0.26862954_f32, 0.26130258, 0.27577711];

    let mut output = Vec::with_capacity((target_size.0 * target_size.1 * 3) as usize);

    for y in 0..target_size.1 {
        for x in 0..target_size.0 {
            let pixel = rgb.get_pixel(x, y);
            for c in 0..3 {
                let val = pixel[c] as f32 / 255.0;
                let normalized = (val - mean[c]) / std[c];
                output.push(normalized);
            }
        }
    }

    Ok(output)
}

// ---------------------------------------------------------------------------
// Vision Encoder Trait
// ---------------------------------------------------------------------------

pub trait VisionEncoder: Send + Sync {
    fn input_size(&self) -> (u32, u32);
    fn embed_dim(&self) -> usize;
    fn num_image_tokens(&self) -> usize;
    fn encode(&self, images: &[Vec<f32>]) -> Result<Vec<f32>>;
}

#[derive(Debug, Clone)]
pub struct VisionEncoderConfig {
    pub model_path: String,
    pub input_size: (u32, u32),
    pub embed_dim: usize,
    pub num_image_tokens: usize,
}

impl Default for VisionEncoderConfig {
    fn default() -> Self {
        Self {
            model_path: "models/clip_vit_l14.rknn".to_string(),
            input_size: (224, 224),
            embed_dim: 1024,
            num_image_tokens: 256,
        }
    }
}

// ---------------------------------------------------------------------------
// Stub Vision Encoder
// ---------------------------------------------------------------------------

pub struct StubVisionEncoder {
    config: VisionEncoderConfig,
}

impl StubVisionEncoder {
    pub fn new(config: VisionEncoderConfig) -> Self {
        Self { config }
    }
}

impl VisionEncoder for StubVisionEncoder {
    fn input_size(&self) -> (u32, u32) {
        self.config.input_size
    }

    fn embed_dim(&self) -> usize {
        self.config.embed_dim
    }

    fn num_image_tokens(&self) -> usize {
        self.config.num_image_tokens
    }

    fn encode(&self, images: &[Vec<f32>]) -> Result<Vec<f32>> {
        let n_images = images.len();
        let total_size = n_images * self.config.num_image_tokens * self.config.embed_dim;
        let mut embeddings = vec![0.0f32; total_size];

        for (i, img) in images.iter().enumerate() {
            let sum: f32 = img.iter().sum();
            for j in 0..self.config.num_image_tokens {
                for k in 0..self.config.embed_dim {
                    let idx = i * self.config.num_image_tokens * self.config.embed_dim 
                        + j * self.config.embed_dim + k;
                    embeddings[idx] = (sum + j as f32 * 0.01 + k as f32 * 0.001).sin();
                }
            }
        }

        Ok(embeddings)
    }
}

// ---------------------------------------------------------------------------
// High-level Multimodal Input Builder
// ---------------------------------------------------------------------------

use rkllm_api_sys::{
    RKLLMInput, RKLLMInputType_RKLLM_INPUT_MULTIMODAL, RKLLMMultiModalInput,
};

pub fn build_multimodal_input(
    prompt: &str,
    images_base64: &[String],
    vision_encoder: &dyn VisionEncoder,
) -> Result<RKLLMInput> {
    if images_base64.is_empty() {
        let c_prompt = std::ffi::CString::new(prompt)?;
        return Ok(RKLLMInput {
            role: std::ptr::null(),
            enable_thinking: false,
            input_type: rkllm_api_sys::RKLLMInputType_RKLLM_INPUT_PROMPT,
            __bindgen_anon_1: rkllm_api_sys::RKLLMInput__bindgen_ty_1 {
                prompt_input: c_prompt.into_raw(),
            },
        });
    }

    let mut preprocessed_images = Vec::new();
    for img_b64 in images_base64 {
        let (img, _format) = decode_base64_image(img_b64)?;
        let preprocessed = preprocess_image(img, vision_encoder.input_size())?;
        preprocessed_images.push(preprocessed);
    }

    let embeddings = vision_encoder.encode(&preprocessed_images)?;

    let c_prompt = std::ffi::CString::new(prompt)?;
    let n_images = images_base64.len();
    let n_tokens = vision_encoder.num_image_tokens();

    let input = RKLLMInput {
        role: std::ptr::null(),
        enable_thinking: false,
        input_type: RKLLMInputType_RKLLM_INPUT_MULTIMODAL,
        __bindgen_anon_1: rkllm_api_sys::RKLLMInput__bindgen_ty_1 {
            multimodal_input: RKLLMMultiModalInput {
                prompt: c_prompt.into_raw(),
                image: rkllm_api_sys::RKLLMMultiModalInput__bindgen_ty_1 {
                    image_embed: embeddings.as_ptr() as *mut f32,
                    n_image_tokens: n_tokens,
                    n_image: n_images,
                    image_start: std::ptr::null(),
                    image_end: std::ptr::null(),
                    image_content: std::ptr::null(),
                    image_width: vision_encoder.input_size().0 as usize,
                    image_height: vision_encoder.input_size().1 as usize,
                },
                video: rkllm_api_sys::RKLLMMultiModalInput__bindgen_ty_2 {
                    video_embed: std::ptr::null_mut(),
                    n_frame_tokens: 0,
                    n_frame_per_video: 0,
                    n_video: 0,
                    video_start: std::ptr::null(),
                    video_end: std::ptr::null(),
                    video_content: std::ptr::null(),
                    frame_width: 0,
                    frame_height: 0,
                },
            },
        },
    };

    std::mem::forget(embeddings);
    std::mem::forget(preprocessed_images);

    Ok(input)
}
