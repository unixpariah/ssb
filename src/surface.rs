use crate::{config::Config, util::helpers::combine_images, Position};
use cairo::{Context, ImageSurface};
use image::{imageops, DynamicImage};
use smithay_client_toolkit::{
    output::OutputInfo,
    shell::{wlr_layer::LayerSurface, WaylandSurface},
    shm::slot::Buffer,
};

#[derive(Debug)]
pub struct Surface {
    pub output_info: OutputInfo,
    pub layer_surface: LayerSurface,
    pub width: i32,
    pub background: DynamicImage,
}

impl Surface {
    #[inline]
    pub fn draw(
        &mut self,
        config: &Config,
        module_info: &[crate::ModuleData],
        buffer: &Buffer,
        canvas: &mut [u8],
    ) -> Result<(), Box<dyn crate::Error>> {
        let width = self.width;
        let height = config.height;

        let (left_imgs, center_imgs, mut right_imgs) = module_info.iter().fold(
            (Vec::new(), Vec::new(), Vec::new()),
            |(mut left_imgs, mut center_imgs, mut right_imgs), info| {
                let img = &info.cache;
                match info.position {
                    Position::Left => left_imgs.push(img),
                    Position::Center => center_imgs.push(img),
                    Position::Right => right_imgs.push(img),
                };
                (left_imgs, center_imgs, right_imgs)
            },
        );
        right_imgs.reverse();

        let left = combine_images(&left_imgs);
        let center = combine_images(&center_imgs);
        let right = combine_images(&right_imgs);

        let mut background = self.background.clone(); // Can't overwrite the background so we clone it
        imageops::overlay(&mut background, &left, 0, 0);
        imageops::overlay(
            &mut background,
            &center,
            width as i64 / 2 - center.width() as i64 / 2,
            0,
        );
        imageops::overlay(
            &mut background,
            &right,
            width as i64 - right.width() as i64,
            0,
        );

        background
            .to_rgba8()
            .chunks_exact_mut(4)
            .enumerate()
            .for_each(|(i, pixel)| {
                let offset = i * 4;
                let alpha = pixel[3] as f32 / 255.0;
                canvas[offset] = (pixel[0] as f32 * alpha) as u8;
                canvas[offset + 1] = (pixel[1] as f32 * alpha) as u8;
                canvas[offset + 2] = (pixel[2] as f32 * alpha) as u8;
                canvas[offset + 3] = pixel[3];
            });

        let layer = &self.layer_surface;
        layer.wl_surface().damage_buffer(0, 0, width, height);
        layer.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
        layer.wl_surface().commit();

        Ok(())
    }

    pub fn create_background(&mut self, config: &Config) {
        let width = self.width;
        let height = config.height;

        let img_surface = ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
        let context = Context::new(&img_surface).unwrap();
        let background = config.background;
        context.set_source_rgba(
            background[0] as f64 / 255.0,
            background[1] as f64 / 255.0,
            background[2] as f64 / 255.0,
            background[3] as f64 / 255.0,
        );
        _ = context.paint();
        let mut background = Vec::new();
        _ = img_surface.write_to_png(&mut background);
        if let Ok(img) = image::load_from_memory(&background) {
            self.background = img;
        }
    }

    pub fn is_configured(&self) -> bool {
        self.width != 0
    }

    pub fn change_size(&mut self) {
        if let Some((width, _)) = self.output_info.logical_size {
            self.width = width;
        }
    }
}
