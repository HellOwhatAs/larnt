//! Path handling and output.
//!
//! This module provides types for working with 2D/3D paths and outputting
//! them to various formats like PNG and SVG.
//!
//! # Types
//!
//! - [`Path`]: A single path (sequence of [`Vector`] points)
//! - [`Paths`]: A collection of paths
//!
//! # Example
//!
//! ```no_run
//! use ln::{Scene, Cube, Vector};
//!
//! let mut scene = Scene::new();
//! scene.add(Cube::new(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0)));
//!
//! let paths = scene.render(
//!     Vector::new(4.0, 3.0, 2.0),
//!     Vector::new(0.0, 0.0, 0.0),
//!     Vector::new(0.0, 0.0, 1.0),
//!     1024.0, 1024.0, 50.0, 0.1, 10.0, 0.01,
//! );
//!
//! // Output to different formats
//! paths.write_to_png("output.png", 1024.0, 1024.0);
//! paths.write_to_svg("output.svg", 1024.0, 1024.0).unwrap();
//! ```

use crate::bounding_box::Box;
use crate::filter::Filter;
use crate::matrix::Matrix;
use crate::vector::Vector;
use image::{ImageBuffer, Rgb};
use std::io::Write;

/// A single path represented as a sequence of 3D points.
pub type Path = Vec<Vector>;

/// A collection of paths.
///
/// `Paths` is the main output type from rendering. It contains a collection
/// of polylines that can be filtered, transformed, and output to various formats.
///
/// # Example
///
/// ```
/// use ln::{Paths, Vector};
///
/// // Create paths manually
/// let paths = Paths::from_vec(vec![
///     vec![Vector::new(0.0, 0.0, 0.0), Vector::new(1.0, 1.0, 0.0)],
///     vec![Vector::new(1.0, 0.0, 0.0), Vector::new(0.0, 1.0, 0.0)],
/// ]);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Paths {
    /// The collection of paths.
    pub paths: Vec<Path>,
}

impl Paths {
    /// Creates a new empty `Paths` collection.
    pub fn new() -> Self {
        Paths { paths: Vec::new() }
    }

    /// Creates a `Paths` collection from a vector of paths.
    pub fn from_vec(paths: Vec<Path>) -> Self {
        Paths { paths }
    }

    /// Adds a path to this collection.
    pub fn push(&mut self, path: Path) {
        self.paths.push(path);
    }

    /// Extends this collection with paths from another.
    pub fn extend(&mut self, other: Paths) {
        self.paths.extend(other.paths);
    }

    /// Returns the bounding box of all paths.
    pub fn bounding_box(&self) -> Box {
        if self.paths.is_empty() {
            return Box::default();
        }
        let mut bx = path_bounding_box(&self.paths[0]);
        for path in self.paths.iter().skip(1) {
            bx = bx.extend(path_bounding_box(path));
        }
        bx
    }

    /// Applies a transformation matrix to all paths.
    pub fn transform(&self, matrix: &Matrix) -> Paths {
        let paths = self.paths.iter()
            .map(|path| path_transform(path, matrix))
            .collect();
        Paths { paths }
    }

    /// Subdivides paths into smaller segments.
    ///
    /// This is used internally for visibility testing. The `step` parameter
    /// controls the maximum distance between consecutive points.
    pub fn chop(&self, step: f64) -> Paths {
        let paths = self.paths.iter()
            .map(|path| path_chop(path, step))
            .collect();
        Paths { paths }
    }

    /// Filters paths using a custom filter.
    pub fn filter<F: Filter>(&self, f: &F) -> Paths {
        let mut result = Vec::new();
        for path in &self.paths {
            result.extend(path_filter(path, f));
        }
        Paths { paths: result }
    }

    /// Simplifies paths by removing redundant points.
    ///
    /// Uses the Ramer-Douglas-Peucker algorithm to reduce the number of
    /// points while preserving the overall shape.
    pub fn simplify(&self, threshold: f64) -> Paths {
        let paths = self.paths.iter()
            .map(|path| path_simplify(path, threshold))
            .collect();
        Paths { paths }
    }

    /// Converts the paths to an SVG string.
    ///
    /// # Arguments
    ///
    /// * `width` - The SVG width
    /// * `height` - The SVG height
    pub fn to_svg(&self, width: f64, height: f64) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "<svg width=\"{}\" height=\"{}\" version=\"1.1\" baseProfile=\"full\" xmlns=\"http://www.w3.org/2000/svg\">",
            width, height
        ));
        lines.push(format!("<g transform=\"translate(0,{}) scale(1,-1)\">", height));
        for path in &self.paths {
            lines.push(path_to_svg(path));
        }
        lines.push("</g></svg>".to_string());
        lines.join("\n")
    }

    /// Writes the paths to an SVG file.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ln::{Scene, Cube, Vector};
    ///
    /// let mut scene = Scene::new();
    /// scene.add(Cube::new(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0)));
    ///
    /// let paths = scene.render(
    ///     Vector::new(4.0, 3.0, 2.0),
    ///     Vector::new(0.0, 0.0, 0.0),
    ///     Vector::new(0.0, 0.0, 1.0),
    ///     1024.0, 1024.0, 50.0, 0.1, 10.0, 0.01,
    /// );
    ///
    /// paths.write_to_svg("output.svg", 1024.0, 1024.0).unwrap();
    /// ```
    pub fn write_to_svg(&self, path: &str, width: f64, height: f64) -> std::io::Result<()> {
        let svg = self.to_svg(width, height);
        std::fs::write(path, svg)
    }

    /// Writes the paths to a PNG image file.
    ///
    /// Renders the paths as black lines on a white background.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ln::{Scene, Sphere, Vector};
    ///
    /// let mut scene = Scene::new();
    /// scene.add(Sphere::new(Vector::new(0.0, 0.0, 0.0), 1.0));
    ///
    /// let paths = scene.render(
    ///     Vector::new(4.0, 3.0, 2.0),
    ///     Vector::new(0.0, 0.0, 0.0),
    ///     Vector::new(0.0, 0.0, 1.0),
    ///     512.0, 512.0, 50.0, 0.1, 10.0, 0.01,
    /// );
    ///
    /// paths.write_to_png("output.png", 512.0, 512.0);
    /// ```
    pub fn write_to_png(&self, path: &str, width: f64, height: f64) {
        let scale = 1.0;
        let w = (width * scale) as u32;
        let h = (height * scale) as u32;
        
        let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_pixel(w, h, Rgb([255, 255, 255]));
        
        for path_points in &self.paths {
            for i in 0..path_points.len().saturating_sub(1) {
                let p1 = &path_points[i];
                let p2 = &path_points[i + 1];
                draw_line(&mut img, 
                    (p1.x * scale) as i32, (h as f64 - p1.y * scale) as i32,
                    (p2.x * scale) as i32, (h as f64 - p2.y * scale) as i32,
                    Rgb([0, 0, 0]));
            }
        }
        
        img.save(path).expect("Failed to save PNG");
    }

    /// Writes the paths to a text file.
    ///
    /// Each path is written as a line of semicolon-separated x,y coordinates.
    pub fn write_to_txt(&self, path: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::create(path)?;
        for path_points in &self.paths {
            let line: Vec<String> = path_points.iter()
                .map(|v| format!("{},{}", v.x, v.y))
                .collect();
            writeln!(file, "{}", line.join(";"))?;
        }
        Ok(())
    }
}

fn draw_line(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgb<u8>) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;
    
    let w = img.width() as i32;
    let h = img.height() as i32;
    
    loop {
        if x >= 0 && x < w && y >= 0 && y < h {
            img.put_pixel(x as u32, y as u32, color);
        }
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

fn path_bounding_box(path: &Path) -> Box {
    if path.is_empty() {
        return Box::default();
    }
    let mut bx = Box::new(path[0], path[0]);
    for v in path.iter().skip(1) {
        bx = bx.extend(Box::new(*v, *v));
    }
    bx
}

fn path_transform(path: &Path, matrix: &Matrix) -> Path {
    path.iter().map(|v| matrix.mul_position(*v)).collect()
}

fn path_chop(path: &Path, step: f64) -> Path {
    let mut result = Vec::new();
    for i in 0..path.len().saturating_sub(1) {
        let a = path[i];
        let b = path[i + 1];
        let v = b.sub(a);
        let l = v.length();
        if i == 0 {
            result.push(a);
        }
        let mut d = step;
        while d < l {
            result.push(a.add(v.mul_scalar(d / l)));
            d += step;
        }
        result.push(b);
    }
    result
}

fn path_filter<F: Filter>(path: &Path, f: &F) -> Vec<Path> {
    let mut result = Vec::new();
    let mut current_path = Vec::new();
    
    for v in path {
        if let Some(new_v) = f.filter(*v) {
            current_path.push(new_v);
        } else {
            if current_path.len() > 1 {
                result.push(current_path);
            }
            current_path = Vec::new();
        }
    }
    
    if current_path.len() > 1 {
        result.push(current_path);
    }
    
    result
}

fn path_simplify(path: &Path, threshold: f64) -> Path {
    if path.len() < 3 {
        return path.clone();
    }
    let a = path[0];
    let b = path[path.len() - 1];
    let mut index = 0;
    let mut distance = 0.0_f64;
    
    for (i, p) in path.iter().enumerate().skip(1).take(path.len() - 2) {
        let d = p.segment_distance(a, b);
        if d > distance {
            index = i;
            distance = d;
        }
    }
    
    if distance > threshold {
        let r1 = path_simplify(&path[..=index].to_vec(), threshold);
        let r2 = path_simplify(&path[index..].to_vec(), threshold);
        let mut result = r1[..r1.len()-1].to_vec();
        result.extend(r2);
        result
    } else {
        vec![a, b]
    }
}

fn path_to_svg(path: &Path) -> String {
    let coords: Vec<String> = path.iter()
        .map(|v| format!("{},{}", v.x, v.y))
        .collect();
    let points = coords.join(" ");
    format!("<polyline stroke=\"black\" fill=\"none\" points=\"{}\" />", points)
}
