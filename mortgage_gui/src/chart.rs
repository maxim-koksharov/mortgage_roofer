use mortgage_core::models::LoanResult;
use plotters::prelude::*;
use plotters::style::{FontDesc, FontFamily, FontStyle};

fn caption_font(size: f64) -> FontDesc<'static> {
    FontDesc::new(FontFamily::SansSerif, size, FontStyle::Normal)
}

fn label_font(size: f64) -> FontDesc<'static> {
    FontDesc::new(FontFamily::SansSerif, size, FontStyle::Normal)
}

fn draw_stacked_bar_chart<DB: DrawingBackend>(
    root: &DrawingArea<DB, plotters::coord::Shift>,
    result: &LoanResult,
    caption: &str,
) -> Result<(), String> {
    root.fill(&WHITE)
        .map_err(|e| format!("Fill error: {}", e))?;

    let n = result.payments.len();
    if n == 0 {
        return Err("No payments to chart".to_string());
    }

    let max_y = result
        .payments
        .iter()
        .map(|p| p.principal + p.interest)
        .fold(0.0, f64::max)
        * 1.1;

    if max_y <= 0.0 {
        return Err("Invalid chart data: max_y is zero".to_string());
    }

    let mut chart = ChartBuilder::on(root)
        .caption(caption, caption_font(22.0))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..n as f64, 0.0..max_y)
        .map_err(|e| format!("Chart build error: {}", e))?;

    chart
        .configure_mesh()
        .x_desc("Payment #")
        .y_desc("Amount")
        .label_style(label_font(14.0))
        .draw()
        .map_err(|e| format!("Mesh draw error: {}", e))?;

    let bar_width = 0.8;
    let mut first_principal = true;
    let mut first_interest = true;
    for (idx, p) in result.payments.iter().enumerate() {
        let x = idx as f64;
        let principal_series = chart
            .draw_series(std::iter::once(Rectangle::new(
                [
                    (x - bar_width / 2.0, 0.0),
                    (x + bar_width / 2.0, p.principal),
                ],
                GREEN.filled(),
            )))
            .map_err(|e| format!("Draw principal error: {}", e))?;
        if first_principal {
            principal_series
                .label("Principal")
                .legend(|(x, y)| Rectangle::new([(x, y), (x + 20, y)], GREEN.filled()));
            first_principal = false;
        }
        let interest_series = chart
            .draw_series(std::iter::once(Rectangle::new(
                [
                    (x - bar_width / 2.0, p.principal),
                    (x + bar_width / 2.0, p.principal + p.interest),
                ],
                RED.filled(),
            )))
            .map_err(|e| format!("Draw interest error: {}", e))?;
        if first_interest {
            interest_series
                .label("Interest")
                .legend(|(x, y)| Rectangle::new([(x, y), (x + 20, y)], RED.filled()));
            first_interest = false;
        }
    }

    if let Some(cross_idx) = result.principal_exceeds_interest_at {
        let cross_payment = &result.payments[cross_idx];
        chart
            .draw_series(std::iter::once(Circle::new(
                (
                    cross_idx as f64,
                    cross_payment.principal + cross_payment.interest,
                ),
                6,
                BLUE.filled(),
            )))
            .map_err(|e| format!("Draw circle error: {}", e))?;
        chart
            .draw_series(std::iter::once(Text::new(
                format!("Cross #{} ({})", cross_idx + 1, cross_payment.date),
                (cross_idx as f64, max_y * 0.92),
                label_font(12.0).color(&BLUE),
            )))
            .map_err(|e| format!("Draw text error: {}", e))?;
    }

    chart
        .configure_series_labels()
        .border_style(BLACK)
        .label_font(label_font(14.0))
        .draw()
        .map_err(|e| format!("Labels draw error: {}", e))?;

    Ok(())
}

pub fn generate_stacked_bar_chart_png(result: &LoanResult) -> Result<Vec<u8>, String> {
    use image::ImageEncoder;

    let (width, height) = (900u32, 500u32);
    let mut raw = vec![0u8; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut raw, (width, height)).into_drawing_area();
        draw_stacked_bar_chart(&root, result, "Principal vs Interest (Stacked)")?;
    }

    let mut png_buf = Vec::new();
    {
        let mut cursor = std::io::Cursor::new(&mut png_buf);
        image::codecs::png::PngEncoder::new(&mut cursor)
            .write_image(&raw, width, height, image::ColorType::Rgb8)
            .map_err(|e| format!("PNG encode error: {}", e))?;
    }
    Ok(png_buf)
}
