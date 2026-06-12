use mortgage_core::models::LoanResult;
use plotters::prelude::*;

pub fn generate_stacked_bar_chart_svg(result: &LoanResult) -> String {
    let mut svg_data = String::new();
    {
        let root = SVGBackend::with_string(&mut svg_data, (900, 500)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let n = result.payments.len();
        let max_y = result
            .payments
            .iter()
            .map(|p| p.principal + p.interest)
            .fold(0.0, f64::max)
            * 1.1;

        let mut chart = ChartBuilder::on(&root)
            .caption("Principal vs Interest (Stacked)", ("sans-serif", 28))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..n as f64, 0.0..max_y)
            .unwrap();

        chart.configure_mesh().draw().unwrap();

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
                .unwrap();
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [
                        (x - bar_width / 2.0, p.principal),
                        (x + bar_width / 2.0, p.principal + p.interest),
                    ],
                    RED.filled(),
                )))
                .unwrap();
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
                .unwrap();
            chart
                .draw_series(std::iter::once(Text::new(
                    format!("Cross #{} ({})", cross_idx + 1, cross_payment.date),
                    (cross_idx as f64, max_y * 0.92),
                    ("sans-serif", 12).into_font().color(&BLUE),
                )))
                .unwrap();
        }

        chart
            .configure_series_labels()
            .border_style(BLACK)
            .draw()
            .unwrap();
    }
    svg_data
}

pub fn generate_stacked_bar_chart_png(result: &LoanResult) -> Vec<u8> {
    use image::ImageEncoder;
    let (width, height) = (900u32, 500u32);
    let mut raw = vec![0u8; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut raw, (width, height)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let n = result.payments.len();
        let max_y = result
            .payments
            .iter()
            .map(|p| p.principal + p.interest)
            .fold(0.0, f64::max)
            * 1.1;

        let mut chart = ChartBuilder::on(&root)
            .caption("Principal vs Interest (Stacked)", ("sans-serif", 28))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..n as f64, 0.0..max_y)
            .unwrap();

        chart.configure_mesh().draw().unwrap();

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
                .unwrap();
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [
                        (x - bar_width / 2.0, p.principal),
                        (x + bar_width / 2.0, p.principal + p.interest),
                    ],
                    RED.filled(),
                )))
                .unwrap();
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
                .unwrap();
            chart
                .draw_series(std::iter::once(Text::new(
                    format!("Cross #{} ({})", cross_idx + 1, cross_payment.date),
                    (cross_idx as f64, max_y * 0.92),
                    ("sans-serif", 12).into_font().color(&BLUE),
                )))
                .unwrap();
        }

        chart
            .configure_series_labels()
            .border_style(BLACK)
            .draw()
            .unwrap();
    }

    let mut png_buf = Vec::new();
    {
        let mut cursor = std::io::Cursor::new(&mut png_buf);
        image::codecs::png::PngEncoder::new(&mut cursor)
            .write_image(&raw, width, height, image::ColorType::Rgb8)
            .unwrap();
    }
    png_buf
}

pub fn generate_balance_line_chart_svg(result: &LoanResult) -> String {
    let mut svg_data = String::new();
    {
        let root = SVGBackend::with_string(&mut svg_data, (900, 500)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let n = result.payments.len();
        let max_y = result
            .payments
            .iter()
            .map(|p| p.remaining_balance)
            .fold(0.0, f64::max)
            * 1.1;

        let mut chart = ChartBuilder::on(&root)
            .caption("Remaining Balance Over Time", ("sans-serif", 28))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..n as f64, 0.0..max_y)
            .unwrap();

        chart.configure_mesh().draw().unwrap();

        let points: Vec<(f64, f64)> = result
            .payments
            .iter()
            .enumerate()
            .map(|(i, p)| (i as f64, p.remaining_balance))
            .collect();

        chart.draw_series(LineSeries::new(points, &BLUE)).unwrap();

        chart
            .configure_series_labels()
            .border_style(BLACK)
            .draw()
            .unwrap();
    }
    svg_data
}

pub fn generate_overlay_chart_svg(result: &LoanResult) -> String {
    let mut svg_data = String::new();
    {
        let root = SVGBackend::with_string(&mut svg_data, (900, 500)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let n = result.payments.len();
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

        let mut chart = ChartBuilder::on(&root)
            .caption("Amortization Overview", ("sans-serif", 28))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..n as f64, 0.0..max_y)
            .unwrap();

        chart.configure_mesh().draw().unwrap();

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
            .unwrap();
        chart
            .draw_series(LineSeries::new(interest_pts, &RED))
            .unwrap();
        chart
            .draw_series(LineSeries::new(balance_pts, &BLUE))
            .unwrap();

        chart
            .configure_series_labels()
            .border_style(BLACK)
            .draw()
            .unwrap();
    }
    svg_data
}
