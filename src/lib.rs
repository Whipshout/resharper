#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use image::imageops::{overlay, FilterType};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};
use napi::{bindgen_prelude::*, Error, JsObject, Result, Status};

pub enum ResizeMode {
  Width(u32),
  Height(u32),
  Scale(f32),
}

pub enum OffsetMode {
  Pixel(i64, i64),
  Percent(f32, f32),
  Center,
}

#[napi(object)]
pub struct BuildCompositedImageOptions {
  pub background_color: Vec<u8>,
  pub resize_mode: Option<JsObject>,
  pub offset_mode: Option<JsObject>,
}

impl BuildCompositedImageOptions {
  pub fn get_resize_mode(&self) -> Result<Option<ResizeMode>> {
    if let Some(resize_mode_obj) = &self.resize_mode {
      let type_str: String = resize_mode_obj.get_named_property::<String>("type")?;

      let resize_mode = match type_str.as_str() {
        "Width" => {
          let value: u32 = resize_mode_obj.get_named_property::<u32>("value")?;
          ResizeMode::Width(value)
        }
        "Height" => {
          let value: u32 = resize_mode_obj.get_named_property::<u32>("value")?;
          ResizeMode::Height(value)
        }
        "Scale" => {
          let value: f64 = resize_mode_obj.get_named_property::<f64>("value")?;
          ResizeMode::Scale(value as f32)
        }
        _ => {
          return Err(Error::new(
            Status::InvalidArg,
            "Invalid ResizeMode type".to_string(),
          ))
        }
      };
      Ok(Some(resize_mode))
    } else {
      Ok(None)
    }
  }

  pub fn get_offset_mode(&self) -> Result<OffsetMode> {
    if let Some(offset_mode_obj) = &self.offset_mode {
      let type_str: String = offset_mode_obj.get_named_property::<String>("type")?;

      let offset_mode = match type_str.as_str() {
        "Pixel" => {
          let value: Vec<i64> = offset_mode_obj.get_named_property::<Vec<i64>>("value")?;
          OffsetMode::Pixel(value[0], value[1])
        }
        "Percent" => {
          let value: Vec<f64> = offset_mode_obj.get_named_property::<Vec<f64>>("value")?;
          OffsetMode::Percent(value[0] as f32, value[1] as f32)
        }
        "Center" => OffsetMode::Center,
        _ => {
          return Err(Error::new(
            Status::InvalidArg,
            "Invalid OffsetMode type".to_string(),
          ))
        }
      };
      Ok(offset_mode)
    } else {
      Ok(OffsetMode::Center)
    }
  }
}

#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}

#[napi]
pub fn build_composited_image(
  product_buffer: Buffer,
  overlay_buffer: Buffer,
  options: BuildCompositedImageOptions,
) {
  let background_color: Vec<u8> = options.background_color.clone();

  let product_image = image::load_from_memory(&product_buffer).unwrap();
  let overlay_image = image::load_from_memory(&overlay_buffer).unwrap();

  let product_image = match options.get_resize_mode().unwrap() {
    Some(resize_mode) => resize_image(&product_image, resize_mode),
    None => product_image,
  };

  let (width, height) = overlay_image.dimensions();

  let color = Rgba(background_color.as_slice().try_into().unwrap());
  let mut background: RgbaImage = ImageBuffer::from_pixel(width, height, color);

  let offset = options.get_offset_mode().unwrap();

  compose(&mut background, &product_image, &overlay_image, offset);

  background.save("./result.png").unwrap();
}

fn compose(
  background_img: &mut RgbaImage,
  product_img: &DynamicImage,
  overlay_img: &DynamicImage,
  product_offset: OffsetMode,
) {
  let base_size = background_img.dimensions();

  let product_size = product_img.dimensions();
  let (product_x, product_y) = calculate_position(product_offset, product_size, base_size);
  overlay(background_img, product_img, product_x, product_y);

  let overlay_size = overlay_img.dimensions();
  let overlay_x = ((base_size.0.saturating_sub(overlay_size.0)) / 2) as i64;
  let overlay_y = ((base_size.1.saturating_sub(overlay_size.1)) / 2) as i64;
  overlay(background_img, overlay_img, overlay_x, overlay_y);
}

fn calculate_position(
  offset: OffsetMode,
  extra_size: (u32, u32),
  base_size: (u32, u32),
) -> (i64, i64) {
  match offset {
    OffsetMode::Pixel(x, y) => {
      let x_pos = x - extra_size.0 as i64 / 2;
      let y_pos = y - extra_size.1 as i64 / 2;
      (x_pos, y_pos)
    }
    OffsetMode::Percent(x_percent, y_percent) => {
      let x_pos = (base_size.0 as f32 * x_percent / 100.0) - extra_size.0 as f32 / 2.0;
      let y_pos = (base_size.1 as f32 * y_percent / 100.0) - extra_size.1 as f32 / 2.0;
      (x_pos as i64, y_pos as i64)
    }
    OffsetMode::Center => (
      ((base_size.0.saturating_sub(extra_size.0)) / 2) as i64,
      ((base_size.1.saturating_sub(extra_size.1)) / 2) as i64,
    ),
  }
}

fn resize_image(image: &DynamicImage, mode: ResizeMode) -> DynamicImage {
  let (original_width, original_height) = image.dimensions();

  let (new_width, new_height) = match mode {
    ResizeMode::Width(target_width) => {
      let aspect_ratio = original_height as f32 / original_width as f32;
      let new_height = (target_width as f32 * aspect_ratio) as u32;
      (target_width, new_height)
    }
    ResizeMode::Height(target_height) => {
      let aspect_ratio = original_width as f32 / original_height as f32;
      let new_width = (target_height as f32 * aspect_ratio) as u32;
      (new_width, target_height)
    }
    ResizeMode::Scale(factor) => {
      let new_width = (original_width as f32 * factor) as u32;
      let new_height = (original_height as f32 * factor) as u32;
      (new_width, new_height)
    }
  };

  image.resize_exact(new_width, new_height, FilterType::Lanczos3)
}
