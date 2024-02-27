pub mod image_combiner {
    use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, ImageError, imageops, Rgba};
    use image::imageops::{overlay, resize};

    fn create_round_avatar(avatar: DynamicImage, target_size: u32) -> DynamicImage {
        // Redimensiona el avatar al tamaño deseado antes de aplicar la máscara circular
        let avatar = resize(&avatar, target_size, target_size, imageops::Lanczos3);
        let (width, height) = avatar.dimensions();
        let radius = width as f32 / 2.0;
        let mut mask = ImageBuffer::new(width, height);
        let center = (width as f32 / 2.0, height as f32 / 2.0);

        for (x, y, pixel) in mask.enumerate_pixels_mut() {
            let dx = x as f32 - center.0 + 0.5; // +0.5 para centrar el pixel
            let dy = y as f32 - center.1 + 0.5;
            if dx.powi(2) + dy.powi(2) <= radius.powi(2) {
                *pixel = Rgba([255, 255, 255, 255]);
            } else {
                *pixel = Rgba([0, 0, 0, 0]);
            }
        }

        // Aplica la máscara al avatar redimensionado
        let round_avatar = ImageBuffer::from_fn(width, height, |x, y| {
            let mask_pixel = mask.get_pixel(x, y);
            let avatar_pixel = avatar.get_pixel(x, y);
            if mask_pixel[3] > 0 {
                *avatar_pixel
            } else {
                Rgba([0, 0, 0, 0])
            }
        });

        DynamicImage::ImageRgba8(round_avatar)
    }

    pub fn combine_images(background_path: &str, avatar_path: &str, x: u32, y: u32, target_size: u32) -> Result<DynamicImage, ImageError> {
        let mut background = image::open(background_path)?;
        let avatar = image::open(avatar_path)?;
        let round_avatar = create_round_avatar(avatar, target_size).to_rgba8();

        let adjusted_x = if x >= 10 { x - 10 } else { 0 };
        let adjusted_y = if y >= 10 { y - 10 } else { 0 };

        for (ax, ay, pixel) in round_avatar.enumerate_pixels() {
            let bx = adjusted_x + ax;
            let by = adjusted_y + ay;
            if bx < background.width() && by < background.height() {
                let background_pixel = background.get_pixel(bx, by);

                if background_pixel[3] < 128 {
                    background.put_pixel(bx, by, *pixel);
                }
            }
        }

        // Calcula la escala y redimensiona si es necesario
        let (avatar_width, avatar_height) = round_avatar.dimensions();
        if adjusted_x + avatar_width > background.width() || adjusted_y + avatar_height > background.height() {
            // Si el avatar es demasiado grande, calcula la nueva escala
            let scale_x = (background.width() - adjusted_x) as f64 / avatar_width as f64;
            let scale_y = (background.height() - adjusted_y) as f64 / avatar_height as f64;
            let scale = scale_x.min(scale_y);

            // Calcula las nuevas dimensiones
            let new_width = (avatar_width as f64 * scale) as u32;
            let new_height = (avatar_height as f64 * scale) as u32;

            // Redimensiona el avatar
            let resized_avatar = resize(&round_avatar, new_width, new_height, imageops::FilterType::Lanczos3);
            overlay(&mut background, &DynamicImage::ImageRgba8(resized_avatar), adjusted_x as i64, adjusted_y as i64);
        } else {
            // Si el avatar ya cabe, colócalo directamente
            overlay(&mut background, &round_avatar, adjusted_x as i64, adjusted_y as i64);
        }

        let background_result = DynamicImage::ImageRgba8(background.into());

        Ok(background_result)
    }
}