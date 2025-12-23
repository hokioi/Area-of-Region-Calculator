/*
PLAN 
THE GRAPH WILL BE ABOUT FINDING THE AREA OF AN IMAGE
THE PROGRAM WILL PLOT POINTS ON AN IMAGE (BIRDS EYE VIEW)
THEN IT WILL CONNECT THEM TO A GRAPH
THEN IT WILL FIND THE AREA OF THE SHAPE
THEN IT WILL PRINT THE AREA
*/
use std::io::{self, Write};
use std::path::Path;
use image::{open, DynamicImage, Rgba};
use imageproc::drawing::{draw_polygon_mut, draw_filled_circle_mut};
use imageproc::point::Point;
use std::f64::consts::PI;

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

fn parse_pair(s: &str) -> Option<(f64, f64)> {
    let mut parts = s.split_whitespace();
    let a = parts.next()?.replace(',', ".");
    let b = parts.next()?.replace(',', ".");
    let x = a.parse::<f64>().ok()?;
    let y = b.parse::<f64>().ok()?;
    Some((x, y))
}

fn polygon_area(points: &[(f64, f64)]) -> f64 {
    // Shoelace formula
    let n = points.len();
    if n < 3 { return 0.0; }
    let mut sum = 0.0;
    for i in 0..n {
        let (x_i, y_i) = points[i];
        let (x_j, y_j) = points[(i + 1) % n];
        sum += x_i * y_j - x_j * y_i;
    }
    sum.abs() * 0.5
}

fn main() {
    println!("Image polygon area calculator");
    println!("Open an image file (screenshot from Google Earth) and provide scale to get real-world area.");
    print!("Enter path to image file: ");
    io::stdout().flush().unwrap();
    let image_path = match read_line_trimmed() {
        Some(p) => p,
        None => {
            println!("No input — exiting.");
            return;
        }
    };

    if !Path::new(&image_path).exists() {
        println!("File not found: {}", image_path);
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
    println!("Loaded image: {} ({} x {})", image_path, width, height);

    println!("\nScale input options:");
    println!("1) Enter scale directly (units per pixel, e.g. meters per pixel)");
    println!("2) Derive scale from two pixel points and a known real-world distance between them");
    print!("Choose 1 or 2: ");
    io::stdout().flush().unwrap();
    let mode = read_line_trimmed().unwrap_or_default();

    let units_per_pixel = if mode.trim() == "2" {
        println!("Enter first pixel point as: x y");
        print!("first point: ");
        io::stdout().flush().unwrap();
        let p1 = match read_line_trimmed().and_then(|s| parse_pair(&s)) {
            Some(p) => p,
            None => { println!("Invalid point."); return; }
        };
        print!("second point: ");
        io::stdout().flush().unwrap();
        let p2 = match read_line_trimmed().and_then(|s| parse_pair(&s)) {
            Some(p) => p,
            None => { println!("Invalid point."); return; }
        };
        print!("Enter real-world distance between those two points (in meters, enter number): ");
        io::stdout().flush().unwrap();
        let dist_real = match read_line_trimmed().map(|s| s.replace(',', ".")).and_then(|s| s.parse::<f64>().ok()) {
            Some(d) if d > 0.0 => d,
            _ => { println!("Invalid distance."); return; }
        };
        let dx = p1.0 - p2.0;
        let dy = p1.1 - p2.1;
        let pixel_dist = (dx*dx + dy*dy).sqrt();
        if pixel_dist == 0.0 {
            println!("Points are identical.");
            return;
        }
        println!("Pixel distance between reference points: {:.3}", pixel_dist);
        dist_real / pixel_dist // units per pixel (e.g. meters/pixel)
    } else {
        print!("Enter units per pixel (e.g. meters per pixel). Use decimal point: ");
        io::stdout().flush().unwrap();
        match read_line_trimmed().map(|s| s.replace(',', ".")).and_then(|s| s.parse::<f64>().ok()) {
            Some(v) if v > 0.0 => v,
            _ => { println!("Invalid scale."); return; }
        }
    };

    println!("\nNow enter polygon points in pixel coordinates (x y). One point per line.");
    println!("Enter a blank line when finished. Example: 120 300");
    let mut points: Vec<(f64,f64)> = Vec::new();
    loop {
        print!("point #{}: ", points.len() + 1);
        io::stdout().flush().unwrap();
        match read_line_trimmed() {
            Some(line) => {
                if let Some(p) = parse_pair(&line) {
                    if p.0 < 0.0 || p.1 < 0.0 || p.0 > width as f64 || p.1 > height as f64 {
                        println!("Warning: point outside image bounds.");
                    }
                    points.push(p);
                } else {
                    println!("Couldn't parse point. Use: x y");
                }
            }
            None => break,
        }
    }

    if points.len() < 3 {
        println!("At least 3 points required to form a polygon.");
        return;
    }

    let area_pixels = polygon_area(&points);
    let area_real = area_pixels * units_per_pixel * units_per_pixel;

    println!("\nResults:");
    println!("Polygon vertices: {}", points.len());
    println!("Pixel area (px²): {:.3}", area_pixels);
    println!("Real area (units²): {:.6}", area_real);

    // draw overlay and save
    let mut out_img = image::DynamicImage::ImageRgba8(img).to_rgba8();
    // draw filled polygon with semi-transparent color
    let poly_pts: Vec<Point<i32>> = points.iter().map(|&(x,y)| Point::new(x.round() as i32, y.round() as i32)).collect();
    let color = Rgba([255u8, 0u8, 0u8, 100u8]); // semi-transparent red
    draw_polygon_mut(&mut out_img, &poly_pts, color);
    // draw vertices as small filled circles
    for &(x,y) in &points {
        draw_filled_circle_mut(&mut out_img, (x.round() as i32, y.round() as i32), 4, Rgba([0u8,255u8,0u8,255u8]));
    }

    let out_path = "output_with_polygon.png";
    if let Err(e) = out_img.save(out_path) {
        println!("Failed to save overlay image: {}", e);
    } else {
        println!("Overlay saved to {}", out_path);
    }

    println!("Done.");
}
