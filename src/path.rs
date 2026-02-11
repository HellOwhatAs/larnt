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
//! use larnt::{Scene, Cube, Vector};
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
#[cfg(feature = "image")]
use image::{ImageBuffer, Pixel, Rgb};
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
/// use larnt::{Paths, Vector};
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
        let paths = self
            .paths
            .iter()
            .map(|path| path_transform(path, matrix))
            .collect();
        Paths { paths }
    }

    /// Subdivides paths into smaller segments.
    ///
    /// This is used internally for visibility testing. The `step` parameter
    /// controls the maximum distance between consecutive points.
    pub fn chop(&self, step: f64) -> Paths {
        let paths = self
            .paths
            .iter()
            .map(|path| path_chop(path, step))
            .collect();
        Paths { paths }
    }

    pub fn chop_adaptive(&self, screen_mat: &Matrix, width: f64, height: f64, step: f64) -> Paths {
        let paths = self
            .paths
            .iter()
            .map(|path| path_chop_adaptive(path, screen_mat, width, height, step))
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
        let paths = self
            .paths
            .iter()
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
        lines.push(format!(
            "<g transform=\"translate(0,{}) scale(1,-1)\">",
            height
        ));
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
    /// use larnt::{Scene, Cube, Vector};
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

    /// Converts the paths to an ImageBuffer.
    ///
    /// # Arguments
    ///
    /// * `width` - The image width
    /// * `height` - The image height
    /// * `linewidth` - The thickness of the lines in pixels
    #[cfg(feature = "image")]
    pub fn to_image(
        &self,
        width: f64,
        height: f64,
        linewidth: f64,
    ) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let scale = 1.0;
        let w = (width * scale) as u32;
        let h = (height * scale) as u32;

        let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::from_pixel(w, h, Rgb([255, 255, 255]));

        for path_points in &self.paths {
            for i in 0..path_points.len().saturating_sub(1) {
                let p1 = &path_points[i];
                let p2 = &path_points[i + 1];
                draw_line(
                    &mut img,
                    p1.x * scale,
                    h as f64 - p1.y * scale,
                    p2.x * scale,
                    h as f64 - p2.y * scale,
                    linewidth,
                    Rgb([0, 0, 0]),
                );
            }
        }

        img
    }

    /// Writes the paths to a PNG image file.
    ///
    /// Renders the paths as black lines on a white background.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use larnt::{Scene, Sphere, Vector};
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
    #[cfg(feature = "png")]
    pub fn write_to_png(&self, path: &str, width: f64, height: f64) {
        let img = self.to_image(width, height, 2.5);
        img.save(path).expect("Failed to save PNG");
    }

    /// Writes the paths to a text file.
    ///
    /// Each path is written as a line of semicolon-separated x,y coordinates.
    pub fn write_to_txt(&self, path: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::create(path)?;
        for path_points in &self.paths {
            let line: Vec<String> = path_points
                .iter()
                .map(|v| format!("{},{}", v.x, v.y))
                .collect();
            writeln!(file, "{}", line.join(";"))?;
        }
        Ok(())
    }
}

#[cfg(feature = "image")]
fn draw_line(
    img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    width: f64,
    color: Rgb<u8>,
) {
    let w = img.width() as i32;
    let h = img.height() as i32;
    let radius = width / 2.0;

    let min_x = (x0.min(x1) - radius - 1.0).floor() as i32;
    let max_x = (x0.max(x1) + radius + 1.0).ceil() as i32;
    let min_y = (y0.min(y1) - radius - 1.0).floor() as i32;
    let max_y = (y0.max(y1) + radius + 1.0).ceil() as i32;

    let min_x = min_x.max(0);
    let max_x = max_x.min(w);
    let min_y = min_y.max(0);
    let max_y = max_y.min(h);

    let dx = x1 - x0;
    let dy = y1 - y0;
    let line_len_sq = dx * dx + dy * dy;

    for y in min_y..max_y {
        for x in min_x..max_x {
            let px = x as f64;
            let py = y as f64;

            let t = if line_len_sq == 0.0 {
                0.0
            } else {
                let dot = (px - x0) * dx + (py - y0) * dy;
                (dot / line_len_sq).clamp(0.0, 1.0)
            };

            let closest_x = x0 + t * dx;
            let closest_y = y0 + t * dy;

            let dist_x = px - closest_x;
            let dist_y = py - closest_y;
            let dist = (dist_x * dist_x + dist_y * dist_y).sqrt();

            let alpha = if dist <= radius - 0.5 {
                1.0
            } else if dist >= radius + 0.5 {
                0.0
            } else {
                1.0 - (dist - (radius - 0.5))
            };

            if alpha > 0.0 {
                let pixel_x = x as u32;
                let pixel_y = y as u32;
                let bg_pixel = img.get_pixel(pixel_x, pixel_y);
                let bg_channels = bg_pixel.channels();
                let fg_channels = color.channels();
                let mut new_channels = [0u8; 3];

                for i in 0..3 {
                    let bg_val = bg_channels[i] as f64;
                    let fg_val = fg_channels[i] as f64;
                    new_channels[i] = (bg_val * (1.0 - alpha) + fg_val * alpha) as u8;
                }

                img.put_pixel(pixel_x, pixel_y, *Rgb::from_slice(&new_channels));
            }
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

fn path_chop_adaptive(
    path: &Path,
    screen_mat: &Matrix,
    width: f64,
    height: f64,
    step: f64,
) -> Path {
    let mut result = vec![path[0]];
    let step_sq = step.powi(2);
    for i in 0..path.len().saturating_sub(1) {
        let (a, b) = (path[i], path[i + 1]);
        recursive_subdivide(a, b, screen_mat, width, height, step_sq, &mut result);
    }
    result
}

fn recursive_subdivide(
    a: Vector,
    b: Vector,
    screen_mat: &Matrix,
    width: f64,
    height: f64,
    step_sq: f64,
    result: &mut Vec<Vector>,
) {
    let (sa, sb) = (screen_mat.mul_position_w(a), screen_mat.mul_position_w(b));
    if (sa.x < 0.0 && sb.x < 0.0
        || sa.y < 0.0 && sb.y < 0.0
        || sa.x > width && sb.x > width
        || sa.y > height && sb.y > height)
        || sa.distance_squared(sb) < step_sq
        || a.distance_squared(b) < crate::common::EPS
    {
        result.push(b);
    } else {
        let mid = a.add(b).mul_scalar(0.5);
        recursive_subdivide(a, mid, screen_mat, width, height, step_sq, result);
        recursive_subdivide(mid, b, screen_mat, width, height, step_sq, result);
    }
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
        let mut result = r1[..r1.len() - 1].to_vec();
        result.extend(r2);
        result
    } else {
        vec![a, b]
    }
}

fn path_to_svg(path: &Path) -> String {
    let coords: Vec<String> = path.iter().map(|v| format!("{},{}", v.x, v.y)).collect();
    let points = coords.join(" ");
    format!(
        "<polyline stroke=\"black\" fill=\"none\" points=\"{}\" />",
        points
    )
}
