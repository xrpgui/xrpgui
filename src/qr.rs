use egui::{ColorImage, Color32, TextureHandle, TextureOptions};

pub fn render_qr_texture(ctx: &egui::Context, data: &str, pixels: u32) -> Option<TextureHandle> {
    let code = qrcode::QrCode::new(data.as_bytes()).ok()?;
    let dim = code.width() as usize;
    let module = (pixels as usize / dim).max(1);
    let size = dim * module;

    let mut img = vec![Color32::WHITE; size * size];
    for (y, row) in code.into_colors().chunks(dim).enumerate() {
        for (x, color) in row.iter().enumerate() {
            if *color == qrcode::Color::Dark {
                for dy in 0..module {
                    for dx in 0..module {
                        let py = y * module + dy;
                        let px = x * module + dx;
                        img[py * size + px] = Color32::BLACK;
                    }
                }
            }
        }
    }

    let color_image = ColorImage::from_rgba_unmultiplied(
        [size, size],
        &img.iter().flat_map(|c| c.to_array()).collect::<Vec<_>>(),
    );
    Some(ctx.load_texture("qr_code", color_image, TextureOptions::NEAREST))
}
