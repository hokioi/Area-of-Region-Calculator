use std::path::Path;
use std::io::{self, Write};

use image::{open, Rgba};
use imageproc::drawing::{draw_polygon_mut, draw_filled_circle_mut};
use imageproc::point::Point;
use rfd::FileDialog;

/// Parse "x y" into (f64, f64)
fn parse_pair(s: &str) -> Option<(f64, f64)> {
    let mut parts = s.split_whitespace();
    let a = parts.next()?.replace(',', ".");
    let b = parts.next()?.replace(',', ".");
    let x = a.parse::<f64>().ok()?;
    let y = b.parse::<f64>().ok()?;
    Some((x, y))
}

/// Shoelace formula for polygon area
fn polygon_area(points: &[(f64, f64)]) -> f64 {
    let n = points.len();
    if n < 3 { return 0.0; }
    let mut sum = 0.0;
    for i in 0..n {
        let (x1, y1) = points[i];
        let (x2, y2) = points[(i+1) % n];
        sum += x1 * y2 - x2 * y1;
    }
    sum.abs() * 0.5
}

/// Read trimmed input line
fn read_line_trimmed() -> Option<String> {
    let mut s = String::new();
    match io::stdin().read_line(&mut s) {
        Ok(0) => None,
        Ok(_) => { 
            let t = s.trim().to_string();
            if t.is_empty() { None } else { Some(t) }
        }
        Err(_) => None,
    }
}

fn main() {
    println!("Image Polygon Area Calculator (Panic-Free)");
    println!("------------------------------------------");

    // --- FILE PICKER ---
    let file_path = FileDialog::new()
        .add_filter("Images", &["png", "jpg", "jpeg"])
        .set_title("Select an image")
        .pick_file();

    let image_path = match file_path {
        Some(path) => path,
        None => {
            println!("No file selected. Exiting.");
            return;
        }
    };

    if !Path::new(&image_path).exists() {
        println!("File not found: {:?}", image_path);
        return;
    }

    let img = match open(&image_path) {
        Ok(i) => i.to_rgba8(),
        Err(e) => {
            println!("Failed to open image: {}", e);
            return;
        }
    };

    let (width, height) = (img.width(), img.height());
    println!("Loaded image: {:?} ({} x {})", image_path, width, height);

    // --- SCALE INPUT ---
    println!("\nScale options:");
    println!("1) Enter units per pixel directly");
    println!("2) Compute scale from two reference points");

    print!("Choose 1 or 2: ");
    io::stdout().flush().unwrap();
    let mode = read_line_trimmed().unwrap_or_default();

    let units_per_pixel = if mode == "2" {
        print!("First pixel point (x y): "); io::stdout().flush().unwrap();
        let p1 = match read_line_trimmed().and_then(|s| parse_pair(&s)) {
            Some(p) => p,
            None => { println!("Invalid point."); return; }
        };

        print!("Second pixel point (x y): "); io::stdout().flush().unwrap();
        let p2 = match read_line_trimmed().and_then(|s| parse_pair(&s)) {
            Some(p) => p,
            None => { println!("Invalid point."); return; }
        };

        print!("Real-world distance between points: "); io::stdout().flush().unwrap();
        let dist_real = match read_line_trimmed().and_then(|s| s.replace(',', ".").parse::<f64>().ok()) {
            Some(d) if d > 0.0 => d,
            _ => { println!("Invalid distance."); return; }
        };

        let dx = p1.0 - p2.0;
        let dy = p1.1 - p2.1;
        let pixel_dist = (dx*dx + dy*dy).sqrt();
        if pixel_dist == 0.0 { println!("Points are identical."); return; }

        dist_real / pixel_dist
    } else {
        print!("Units per pixel: "); io::stdout().flush().unwrap();
        match read_line_trimmed().and_then(|s| s.replace(',', ".").parse::<f64>().ok()) {
            Some(v) if v > 0.0 => v,
            _ => { println!("Invalid scale."); return; }
        }
    };

    // --- POLYGON INPUT ---
    println!("\nEnter polygon points (x y). Blank line to finish.");
    let mut points: Vec<(f64,f64)> = Vec::new();
    loop {
        print!("Point #{}: ", points.len() + 1); io::stdout().flush().unwrap();
        match read_line_trimmed() {
            Some(line) => {
                if let Some(p) = parse_pair(&line) {
                    // Clip points to image bounds to prevent panics
                    let x = p.0.max(0.0).min(width as f64);
                    let y = p.1.max(0.0).min(height as f64);
                    points.push((x, y));
                } else {
                    println!("Invalid format. Use: x y");
                }
            }
            None => break,
        }
    }

    if points.len() < 3 {
        println!("Need at least 3 points to form a polygon.");
        return;
    }

    // Flip Y axis for correct math
    let flipped: Vec<(f64,f64)> = points.iter().map(|&(x,y)| (x, height as f64 - y)).collect();
    let area_pixels = polygon_area(&flipped);
    let area_real = area_pixels * units_per_pixel * units_per_pixel;

    println!("\nResults");
    println!("Vertices: {}", points.len());
    println!("Pixel area: {:.3}", area_pixels);
    println!("Real area: {:.6}", area_real);

    // --- DRAWING ---
    let mut out_img = img.clone();
    let poly_pts: Vec<Point<i32>> = points.iter().map(|&(x,y)| Point::new(x.round() as i32, y.round() as i32)).collect();

    // Only draw polygon if at least 3 points
    if poly_pts.len() >= 3 {
        draw_polygon_mut(&mut out_img, &poly_pts, Rgba([255,0,0,255]));
    }

    // Draw vertex circles safely
    for &(x, y) in &points {
        let xi = x.round() as i32;
        let yi = y.round() as i32;
        if xi >= 0 && xi < width as i32 && yi >= 0 && yi < height as i32 {
            draw_filled_circle_mut(&mut out_img, (xi, yi), 4, Rgba([0,255,0,255]));
        }
    }

    // Save overlay safely next to input image
    let out_path = image_path.with_file_name("output_with_polygon.png");
    match out_img.save(&out_path) {
        Ok(_) => println!("Overlay saved to {:?}", out_path),
        Err(e) => println!("Failed to save overlay image: {}", e),
    }

    println!("Done.");
}
