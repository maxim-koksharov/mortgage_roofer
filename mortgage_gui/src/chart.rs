use mortgage_core::models::LoanResult;
use plotters::prelude::*;
use plotters::style::{FontDesc, FontFamily, FontStyle};

fn default_font(size: f64) -> FontDesc<'static> {
    FontDesc::new(FontFamily::SansSerif, size, FontStyle::Normal)
}

pub fn generate_stacked_bar_chart_svg(result: &LoanResult) -> Result<String, String> {
    let mut svg_data = String::new();
    {
        let root = SVGBackend::with_string(&mut svg_data, (900, 500)).into_drawing_area();
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

        let mut chart = ChartBuilder::on(&root)
            .caption("Principal vs Interest (Stacked)", default_font(28.0))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..n as f64, 0.0..max_y)
            .map_err(|e| format!("Chart build error: {}", e))?;

        chart
            .configure_mesh()
            .draw()
            .map_err(|e| format!("Mesh draw error: {}", e))?;

        let bar_width = 0.8;
        for (idx, p) in result.payments.iter().enumerate() {
            let x = idx as f64;
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [
                        (x - bar_width / 2.0, 0.0),
                        (x + bar_width / 2.0, p.principal),
                    ],
                    GREEN.filled(),
                )))
                .map_err(|e| format!("Draw principal error: {}", e))?;
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [
                        (x - bar_width / 2.0, p.principal),
                        (x + bar_width / 2.0, p.principal + p.interest),
                    ],
                    RED.filled(),
                )))
                .map_err(|e| format!("Draw interest error: {}", e))?;
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
                    default_font(12.0).color(&BLUE),
                )))
                .map_err(|e| format!("Draw text error: {}", e))?;
        }

        chart
            .configure_series_labels()
            .border_style(BLACK)
            .draw()
            .map_err(|e| format!("Labels draw error: {}", e))?;
    }
    Ok(svg_data)
}

pub fn generate_stacked_bar_chart_png(result: &LoanResult) -> Result<Vec<u8>, String> {
    use image::ImageEncoder;
    let (width, height) = (900u32, 500u32);
    let mut raw = vec![0u8; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut raw, (width, height)).into_drawing_area();
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

        let mut chart = ChartBuilder::on(&root)
            .caption("Principal vs Interest (Stacked)", default_font(28.0))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..n as f64, 0.0..max_y)
            .map_err(|e| format!("Chart build error: {}", e))?;

        chart
            .configure_mesh()
            .draw()
            .map_err(|e| format!("Mesh draw error: {}", e))?;

        let bar_width = 0.8;
        for (idx, p) in result.payments.iter().enumerate() {
            let x = idx as f64;
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [
                        (x - bar_width / 2.0, 0.0),
                        (x + bar_width / 2.0, p.principal),
                    ],
                    GREEN.filled(),
                )))
                .map_err(|e| format!("Draw principal error: {}", e))?;
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [
                        (x - bar_width / 2.0, p.principal),
                        (x + bar_width / 2.0, p.principal + p.interest),
                    ],
                    RED.filled(),
                )))
                .map_err(|e| format!("Draw interest error: {}", e))?;
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
                    default_font(12.0).color(&BLUE),
                )))
                .map_err(|e| format!("Draw text error: {}", e))?;
        }

        chart
            .configure_series_labels()
            .border_style(BLACK)
            .draw()
            .map_err(|e| format!("Labels draw error: {}", e))?;
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

pub fn generate_balance_line_chart_svg(result: &LoanResult) -> Result<String, String> {
    let mut svg_data = String::new();
    {
        let root = SVGBackend::with_string(&mut svg_data, (900, 500)).into_drawing_area();
        root.fill(&WHITE)
            .map_err(|e| format!("Fill error: {}", e))?;

        let n = result.payments.len();
        if n == 0 {
            return Err("No payments to chart".to_string());
        }

        let max_y = result
            .payments
            .iter()
            .map(|p| p.remaining_balance)
            .fold(0.0, f64::max)
            * 1.1;

        if max_y <= 0.0 {
            return Err("Invalid chart data: max_y is zero".to_string());
        }

        let mut chart = ChartBuilder::on(&root)
            .caption("Principal vs Interest (Stacked)", default_font(28.0))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..n as f64, 0.0..max_y)
            .map_err(|e| format!("Chart build error: {}", e))?;

        chart
            .configure_mesh()
            .draw()
            .map_err(|e| format!("Mesh draw error: {}", e))?;

        let points: Vec<(f64, f64)> = result
            .payments
            .iter()
            .enumerate()
            .map(|(i, p)| (i as f64, p.remaining_balance))
            .collect();

        chart
            .draw_series(LineSeries::new(points, &BLUE))
            .map_err(|e| format!("Draw line error: {}", e))?;

        chart
            .configure_series_labels()
            .border_style(BLACK)
            .draw()
            .map_err(|e| format!("Labels draw error: {}", e))?;
    }
    Ok(svg_data)
}

pub fn generate_overlay_chart_svg(result: &LoanResult) -> Result<String, String> {
    let mut svg_data = String::new();
    {
        let root = SVGBackend::with_string(&mut svg_data, (900, 500)).into_drawing_area();
        root.fill(&WHITE)
            .map_err(|e| format!("Fill error: {}", e))?;

        let n = result.payments.len();
        if n == 0 {
            return Err("No payments to chart".to_string());
        }

        let max_payment = result
            .payments
            .iter()
            .map(|p| p.principal + p.interest)
            .fold(0.0, f64::max)
            * 1.1;
        let max_balance = result
            .payments
            .iter()
            .map(|p| p.remaining_balance)
            .fold(0.0, f64::max)
            * 1.1;
        let max_y = max_payment.max(max_balance);

        if max_y <= 0.0 {
            return Err("Invalid chart data: max_y is zero".to_string());
        }

        let mut chart = ChartBuilder::on(&root)
            .caption("Amortization Overview", default_font(28.0))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..n as f64, 0.0..max_y)
            .map_err(|e| format!("Chart build error: {}", e))?;

        chart
            .configure_mesh()
            .draw()
            .map_err(|e| format!("Mesh draw error: {}", e))?;

        let principal_pts: Vec<(f64, f64)> = result
            .payments
            .iter()
            .enumerate()
            .map(|(i, p)| (i as f64, p.principal))
            .collect();
        let interest_pts: Vec<(f64, f64)> = result
            .payments
            .iter()
            .enumerate()
            .map(|(i, p)| (i as f64, p.interest))
            .collect();
        let balance_pts: Vec<(f64, f64)> = result
            .payments
            .iter()
            .enumerate()
            .map(|(i, p)| (i as f64, p.remaining_balance))
            .collect();

        chart
            .draw_series(LineSeries::new(principal_pts, &GREEN))
            .map_err(|e| format!("Draw principal line error: {}", e))?;
        chart
            .draw_series(LineSeries::new(interest_pts, &RED))
            .map_err(|e| format!("Draw interest line error: {}", e))?;
        chart
            .draw_series(LineSeries::new(balance_pts, &BLUE))
            .map_err(|e| format!("Draw balance line error: {}", e))?;

        chart
            .configure_series_labels()
            .border_style(BLACK)
            .draw()
            .map_err(|e| format!("Labels draw error: {}", e))?;
    }
    Ok(svg_data)
}
