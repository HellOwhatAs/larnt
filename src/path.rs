//! Path handling and output.
//!
//! This module provides types for working with 2D/3D paths and outputting
//! them to various formats like PNG and SVG.
//!
//! # Types
//!
//! - [`Paths`]: A collection of paths (the primary output type from rendering)
//! - [`NewPath`]: A builder for appending a single path into a [`Paths`] collection
//!
//! # Example
//!
//! ```
//! use larnt::{Cube, Vector, render};
//!
//! let cube = Cube::builder(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0)).build();
//! let paths = render(vec![cube]).eye(Vector::new(4.0, 3.0, 2.0)).call();
//!
//! // Output to different formats
//! paths.write_to_png("output.png", 1024.0, 1024.0).expect("Failed to write PNG");
//! paths.write_to_svg("output.svg", 1024.0, 1024.0).expect("Failed to write PNG");
//! ```

use crate::bounding_box::BBox;
use crate::filter::Filter;
use crate::matrix::Matrix;
use crate::shape::RenderArgs;
use crate::vector::Vector;
use bon::bon;
#[cfg(feature = "image")]
use image::{ImageBuffer, Pixel, Rgba};
use std::collections::HashMap;
use std::f64::consts::PI;
use std::io::Write;

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
/// let mut paths = Paths::new();
/// paths.new_path().extend([Vector::new(0.0, 0.0, 0.0), Vector::new(1.0, 1.0, 0.0)]);
/// let mut new_path = paths.new_path();
/// new_path.push(Vector::new(1.0, 0.0, 0.0));
/// new_path.push(Vector::new(0.0, 1.0, 0.0));
/// drop(new_path);
/// println!("{:?}", paths);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Paths<T> {
    buffer: Vec<T>,
    offsets: Vec<usize>,
}

impl<T> Paths<T> {
    /// Creates a new empty `Paths` collection.
    pub fn new() -> Self {
        Paths {
            buffer: Vec::new(),
            offsets: Vec::new(),
        }
    }

    pub fn with_capacity(total_len: usize, len: usize) -> Self {
        Paths {
            buffer: Vec::with_capacity(total_len),
            offsets: Vec::with_capacity(len),
        }
    }

    pub fn new_path<'a>(&'a mut self) -> NewPath<'a, T> {
        self.offsets.push(self.buffer.len());
        NewPath::new(&mut self.buffer, &mut self.offsets)
    }

    /// Extends this collection with paths from another.
    pub fn extend(&mut self, other: Self) {
        self.offsets
            .extend(other.offsets.into_iter().map(|o| o + self.buffer.len()));
        self.buffer.extend(other.buffer);
    }

    pub fn map<F, U>(self, f: F) -> Paths<U>
    where
        F: FnMut(T) -> U,
    {
        Paths {
            buffer: self.buffer.into_iter().map(f).collect(),
            offsets: self.offsets,
        }
    }

    pub fn get(&self, index: usize) -> Option<&[T]> {
        match (self.offsets.get(index), self.offsets.get(index + 1)) {
            (Some(&i), None) => Some(&self.buffer[i..]),
            (Some(&i1), Some(&i2)) => Some(&self.buffer[i1..i2]),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut [T]> {
        match (self.offsets.get(index), self.offsets.get(index + 1)) {
            (Some(&i), None) => Some(&mut self.buffer[i..]),
            (Some(&i1), Some(&i2)) => Some(&mut self.buffer[i1..i2]),
            _ => None,
        }
    }

    pub fn total_len(&self) -> usize {
        self.buffer.len()
    }

    pub fn len(&self) -> usize {
        self.offsets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }

    pub fn iter_paths(&self) -> impl Iterator<Item = &[T]> {
        (if self.offsets.is_empty() {
            None
        } else {
            Some(
                self.offsets
                    .windows(2)
                    .map(|window| {
                        let (start, end) = (window[0], window[1]);
                        &self.buffer[start..end]
                    })
                    .chain(std::iter::once(
                        &self.buffer[self.offsets.last().copied().unwrap()..],
                    )),
            )
        })
        .into_iter()
        .flatten()
    }
}

impl<'a, T> std::ops::Index<usize> for Paths<T> {
    type Output = [T];
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect(&format!(
            "index out of bounds: the len is {} but the index is {index}",
            self.len()
        ))
    }
}
impl<'a, T> std::ops::IndexMut<usize> for Paths<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let len = self.len();
        self.get_mut(index).expect(&format!(
            "index out of bounds: the len is {len} but the index is {index}",
        ))
    }
}

#[bon]
impl Paths<Vector> {
    /// Converts the paths to an ImageBuffer.
    ///
    /// # Arguments
    ///
    /// * `width` - The image width
    /// * `height` - The image height
    /// * `linewidth` - The thickness of the lines in pixels
    #[cfg(feature = "image")]
    #[builder]
    pub fn to_image(
        &self,
        #[builder(start_fn)] width: f64,
        #[builder(start_fn)] height: f64,
        #[builder(default = 1.0)] linewidth: f64,
        #[builder(default = Rgba([255, 255, 255, 255]))] background: Rgba<u8>,
        #[builder(default = Rgba([0, 0, 0, 255]))] foreground: Rgba<u8>,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let scale = 1.0;
        let w = (width * scale) as u32;
        let h = (height * scale) as u32;

        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(w, h, background);

        for path_points in self.iter_paths() {
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
                    foreground,
                );
            }
        }

        img
    }
}

impl Paths<Vector> {
    /// Returns the bounding box of all paths.
    pub fn bounding_box(&self) -> BBox {
        if self.buffer.is_empty() {
            return BBox::default();
        }
        let mut bx = BBox::new(self.buffer[0], self.buffer[0]);
        for v in self.buffer.iter().skip(1) {
            bx = bx.extend(BBox::new(*v, *v));
        }
        bx
    }

    /// Applies a transformation matrix to all paths.
    pub fn transform(self, matrix: &Matrix) -> Self {
        Self {
            buffer: self
                .buffer
                .into_iter()
                .map(|v| matrix.mul_position(v))
                .collect(),
            offsets: self.offsets,
        }
    }

    /// Subdivides paths into smaller segments.
    ///
    /// This is used internally for visibility testing. The `step` parameter
    /// controls the maximum distance between consecutive points.
    pub fn chop(&self, step: f64) -> Self {
        let mut result = Self::new();
        for path in self.iter_paths() {
            let mut new_path = result.new_path();
            path_chop(path, step, &mut new_path);
        }
        result
    }

    pub fn chop_adaptive(&self, args: &RenderArgs) -> Self {
        let mut result = Self::new();
        for path in self.iter_paths() {
            let mut new_path = result.new_path();
            path_chop_adaptive(
                path,
                &args.screen_mat,
                args.width,
                args.height,
                args.step,
                &mut new_path,
            );
        }
        result
    }

    /// Filters paths using a custom filter.
    pub fn filter<F: Filter>(&self, f: &F) -> Self {
        let mut result = Paths::new();
        for path in self.iter_paths() {
            path_filter(path, f, &mut result);
        }
        result
    }

    /// Simplifies paths by removing redundant points.
    ///
    /// Uses the Ramer-Douglas-Peucker algorithm to reduce the number of
    /// points while preserving the overall shape.
    pub fn simplify(&self, threshold: f64) -> Self {
        let mut result = Paths::new();
        for path in self.iter_paths() {
            path_simplify(path, threshold, &mut result.new_path());
        }
        result
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
        for path in self.iter_paths() {
            lines.push(path_to_svg(path));
        }
        lines.push("</g></svg>".to_string());
        lines.join("\n")
    }

    /// Writes the paths to an SVG file.
    ///
    /// # Example
    ///
    /// ```
    /// use larnt::{Cube, Vector, render};
    ///
    /// let cube = Cube::builder(Vector::new(-1.0, -1.0, -1.0), Vector::new(1.0, 1.0, 1.0)).build();
    /// let paths = render(vec![cube]).eye(Vector::new(4.0, 3.0, 2.0)).call();
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
    /// ```
    /// use larnt::{Sphere, Vector, render};
    ///
    /// let sphere = Sphere::builder(Vector::new(0.0, 0.0, 0.0), 1.0).build();
    /// let paths = render(vec![sphere]).eye(Vector::new(4.0, 3.0, 2.0)).call();
    ///
    /// paths.write_to_png("output.png", 512.0, 512.0).expect("Failed to write PNG");
    /// ```
    #[cfg(feature = "png")]
    pub fn write_to_png(
        &self,
        path: &str,
        width: f64,
        height: f64,
    ) -> Result<(), image::ImageError> {
        let img = self.to_image(width, height).linewidth(2.5).call();
        img.save(path)
    }

    /// Writes the paths to a text file.
    ///
    /// Each path is written as a line of semicolon-separated x,y coordinates.
    pub fn write_to_txt(&self, path: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::create(path)?;
        for path_points in self.iter_paths() {
            let line: Vec<String> = path_points
                .iter()
                .map(|v| format!("{},{}", v.x, v.y))
                .collect();
            writeln!(file, "{}", line.join(";"))?;
        }
        Ok(())
    }
}

impl<T: Copy> Paths<T> {
    pub fn splice<K, FK, FS, I, FC>(
        &self,
        mut head_key_fn: FK,
        mut search_keys_fn: FS,
        mut is_match: FC,
        skip_overlap: bool,
    ) -> Paths<T>
    where
        K: Eq + std::hash::Hash,
        FK: FnMut(&T) -> K,
        FS: FnMut(&T) -> I,
        I: IntoIterator<Item = K>,
        FC: FnMut(&T, &T) -> bool,
    {
        if self.len() == 0 {
            return Paths::new();
        }

        let mut starts: HashMap<K, Vec<Endpoint>> = HashMap::new();
        let mut graph = PathGraph::new(self.len());

        for (id, arr) in self.iter_paths().enumerate() {
            if let Some(first) = arr.first() {
                starts
                    .entry(head_key_fn(first))
                    .or_default()
                    .push(Endpoint::new(id, false));
            }
            if let Some(last) = arr.last() {
                starts
                    .entry(head_key_fn(last))
                    .or_default()
                    .push(Endpoint::new(id, true));
            }
        }

        for id in 0..self.len() {
            let arr = &self[id];
            if arr.is_empty() {
                continue;
            }

            for is_last in [false, true] {
                let ep = Endpoint::new(id, is_last);
                if graph.is_connected(ep) {
                    continue;
                }

                let pt = if is_last {
                    arr.last().unwrap()
                } else {
                    arr.first().unwrap()
                };

                'search: for key in search_keys_fn(pt) {
                    if let Some(candidates) = starts.get_mut(&key) {
                        for i in (0..candidates.len()).rev() {
                            let succ_ep = candidates[i];

                            if succ_ep.path_id() == id {
                                continue;
                            }

                            if graph.is_connected(succ_ep) {
                                candidates.swap_remove(i);
                                continue;
                            }

                            let succ_arr = &self[succ_ep.path_id()];
                            let succ_pt = if succ_ep.is_last() {
                                succ_arr.last().unwrap()
                            } else {
                                succ_arr.first().unwrap()
                            };

                            if is_match(pt, succ_pt) {
                                candidates.swap_remove(i);
                                graph.link(ep, succ_ep);
                                break 'search;
                            }
                        }
                    }
                }
            }
        }

        self.reassemble_paths(&graph, skip_overlap)
    }

    fn reassemble_paths(&self, graph: &PathGraph, skip_overlap: bool) -> Paths<T> {
        let n = self.len();
        let mut paths_out = Paths::with_capacity(self.total_len(), n);
        let mut visited = vec![false; n];

        for is_ring_pass in [false, true] {
            for id in 0..n {
                if visited[id] || self[id].is_empty() {
                    continue;
                }

                let ep_first = Endpoint::new(id, false);
                let ep_last = Endpoint::new(id, true);

                let is_endpoint = !graph.is_connected(ep_first) || !graph.is_connected(ep_last);
                if !is_ring_pass && !is_endpoint {
                    continue;
                }

                let mut entry_ep = if !graph.is_connected(ep_first) {
                    ep_first
                } else {
                    ep_last
                };
                let mut current_id = id;
                let mut new_path = paths_out.new_path();

                loop {
                    visited[current_id] = true;
                    let arr = &self[current_id];
                    let skip_n = if skip_overlap && !new_path.is_empty() {
                        1
                    } else {
                        0
                    };

                    if entry_ep.is_last() {
                        new_path.extend(arr.iter().rev().skip(skip_n).copied());
                    } else {
                        new_path.extend(arr.iter().skip(skip_n).copied());
                    }

                    if let Some(next_entry_ep) = graph.next(entry_ep.opposite()) {
                        let next_id = next_entry_ep.path_id();
                        if visited[next_id] {
                            break;
                        }
                        current_id = next_id;
                        entry_ep = next_entry_ep;
                    } else {
                        break;
                    }
                }
            }
        }
        paths_out
    }

    pub fn splice_exact(&self) -> Paths<T>
    where
        T: std::hash::Hash + std::cmp::Eq + Copy,
    {
        self.splice(|&x| x, |&x| [x], |a, b| a == b, true)
    }
}

pub struct NewPath<'a, T> {
    buffer: &'a mut Vec<T>,
    offsets: &'a mut Vec<usize>,
}

impl<'a, T> NewPath<'a, T> {
    pub fn new(buffer: &'a mut Vec<T>, offsets: &'a mut Vec<usize>) -> Self {
        NewPath { buffer, offsets }
    }

    pub fn push(&mut self, v: T) {
        self.buffer.push(v);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.buffer.len() > self.offsets.last().copied().unwrap_or(0) {
            self.buffer.pop()
        } else {
            None
        }
    }

    pub fn as_slice(&self) -> &[T] {
        let start = self.offsets.last().copied().unwrap_or(0);
        &self.buffer[start..]
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let start = self.offsets.last().copied().unwrap_or(0);
        &mut self.buffer[start..]
    }

    pub fn len(&self) -> usize {
        self.buffer.len() - self.offsets.last().copied().unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: Copy> NewPath<'_, T> {
    pub fn extend_from_slice(&mut self, slice: &[T]) {
        self.buffer.extend_from_slice(slice);
    }
}

impl<T> Extend<T> for NewPath<'_, T> {
    fn extend<T1: IntoIterator<Item = T>>(&mut self, iter: T1) {
        self.buffer.extend(iter);
    }
}

impl<'a, T> Drop for NewPath<'a, T> {
    fn drop(&mut self) {
        if let Some(last_offset) = self.offsets.last().copied()
            && self.buffer.len() == last_offset
        {
            self.offsets.pop();
        }
    }
}

#[cfg(feature = "image")]
fn draw_line(
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    width: f64,
    color: Rgba<u8>,
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
                let mut new_channels = [0u8; 4];

                for i in 0..4 {
                    let bg_val = bg_channels[i] as f64;
                    let fg_val = fg_channels[i] as f64;
                    new_channels[i] = (bg_val * (1.0 - alpha) + fg_val * alpha) as u8;
                }

                img.put_pixel(pixel_x, pixel_y, *Rgba::from_slice(&new_channels));
            }
        }
    }
}

fn path_chop(path: &[Vector], step: f64, new_path: &mut NewPath<Vector>) {
    for i in 0..path.len().saturating_sub(1) {
        let a = path[i];
        let b = path[i + 1];
        let v = b.sub(a);
        let l = v.length();
        if i == 0 {
            new_path.push(a);
        }
        let mut d = step;
        while d < l {
            new_path.push(a.add(v.mul_scalar(d / l)));
            d += step;
        }
        new_path.push(b);
    }
}

fn path_chop_adaptive(
    path: &[Vector],
    screen_mat: &Matrix,
    width: f64,
    height: f64,
    step: f64,
    new_path: &mut NewPath<Vector>,
) {
    if path.is_empty() {
        return;
    }

    new_path.push(path[0]);
    let step_sq = step.powi(2);

    let mut iter = path.iter();
    let mut prev_v = *iter.next().unwrap();
    let mut prev_sv = screen_mat.mul_position_w(prev_v);

    for &curr_v in iter {
        let curr_sv = screen_mat.mul_position_w(curr_v);

        recursive_subdivide(
            ((prev_v, prev_sv), (curr_v, curr_sv)),
            &|(a, _), (b, _)| {
                let mid = a.add(b).mul_scalar(0.5);
                (mid, screen_mat.mul_position_w(mid))
            },
            &|(a, sa), (b, sb)| {
                (sa.x < 0.0 && sb.x < 0.0
                    || sa.y < 0.0 && sb.y < 0.0
                    || sa.x > width && sb.x > width
                    || sa.y > height && sb.y > height)
                    || sa.distance_squared(sb) < step_sq
                    || a.distance_squared(b) < crate::common::EPS
            },
            &mut |(x, _)| new_path.push(x),
        );

        prev_v = curr_v;
        prev_sv = curr_sv;
    }
}

/// Recursively subdivides an arc defined by angles `alpha` and `beta`
/// into a sequence of angles that approximate the arc on screen within a certain step size.
///
/// The arc is defined by a center point `c` and two orthogonal vectors `u` and `v` that
/// define the plane of the arc. The radius `r` determines how far from the center the
/// arc points are. The `screen_mat` is used to project the 3D points onto the screen
/// for distance calculations. The `step_sq` parameter controls how closely the subdivided points
/// approximate the arc on screen, with smaller values resulting in more points for a smoother arc.
fn recursive_arc_subdivide(
    alpha: f64,
    beta: f64,
    r: f64,
    cuv: &(Vector, Vector, Vector),
    screen_mat: &Matrix,
    step_sq: f64,
    collector: &mut impl FnMut(f64),
) {
    let screen_view = |x: f64| {
        screen_mat.mul_position_w(
            (cuv.0)
                .add((cuv.1).mul_scalar(x.cos() * r))
                .add((cuv.2).mul_scalar(x.sin() * r)),
        )
    };
    collector(alpha);
    crate::path::recursive_subdivide(
        ((alpha, screen_view(alpha)), (beta, screen_view(beta))),
        &|(alpha, _), (beta, _)| {
            let mid = (beta + alpha) / 2.0;
            (mid, screen_view(mid))
        },
        &|(alpha, sa), (beta, sb)| {
            let theta = (beta - alpha) / 2.0;
            theta < PI / 180.0
                || sa.distance_squared(sb) * theta / theta.sin() < step_sq && theta < PI / 3.0
        },
        &mut |(x, _)| collector(x),
    );
}

/// Generates a sequence of points along an arc defined by angles `alpha` and `beta`,
/// with adaptive subdivision to ensure smoothness on screen and radius expansion
/// to pass visibility testing.
pub fn adaptive_arc(
    alpha: f64,
    beta: f64,
    r: f64,
    cuv: &(Vector, Vector, Vector),
    screen_mat: &Matrix,
    step_sq: f64,
    new_path: &mut NewPath<Vector>,
) {
    recursive_arc_subdivide(alpha, beta, r, cuv, screen_mat, step_sq, &mut |x| {
        new_path.push(Vector::new(x, 0., 0.))
    });
    let (c, u, v) = cuv;
    let slice = new_path.as_mut_slice();
    let mut prev_r = r;
    for i in 0..slice.len() {
        let cur = slice[i].x;
        let mut max_r = r;
        max_r = max_r.max(prev_r);

        if i + 1 < slice.len() {
            let cos_theta = ((slice[i + 1].x - cur) / 2.0).cos();
            prev_r = r / cos_theta;
            max_r = max_r.max(prev_r);
        }

        slice[i] = c
            .add(u.mul_scalar(cur.cos() * max_r))
            .add(v.mul_scalar(cur.sin() * max_r));
    }
}

/// Similar to `adaptive_arc`, but uses the original radius values
/// instead of expanded values. This can be used for inner arcs.
pub fn adaptive_arc_inner(
    alpha: f64,
    beta: f64,
    r: f64,
    cuv: &(Vector, Vector, Vector),
    screen_mat: &Matrix,
    step_sq: f64,
    new_path: &mut NewPath<Vector>,
) {
    recursive_arc_subdivide(alpha, beta, r, cuv, screen_mat, step_sq, &mut |x| {
        new_path.push(Vector::new(x, 0., 0.))
    });
    let (c, u, v) = cuv;
    new_path.as_mut_slice().iter_mut().for_each(|vector| {
        let cur = vector.x;
        *vector = c
            .add(u.mul_scalar(cur.cos() * r))
            .add(v.mul_scalar(cur.sin() * r));
    });
}

pub fn recursive_subdivide<T: Copy>(
    ab: (T, T),
    divider: &impl Fn(T, T) -> T,
    terminator: &impl Fn(T, T) -> bool,
    collector: &mut impl FnMut(T),
) {
    let (a, b) = ab;
    if terminator(a, b) {
        collector(b);
    } else {
        let mid = divider(a, b);
        recursive_subdivide((a, mid), divider, terminator, collector);
        recursive_subdivide((mid, b), divider, terminator, collector);
    }
}

fn path_filter<F: Filter>(path: &[Vector], f: &F, result: &mut Paths<Vector>) {
    let mut current_path = result.new_path();

    for v in path {
        if let Some(new_v) = f.filter(*v) {
            current_path.push(new_v);
        } else {
            drop(current_path);
            current_path = result.new_path();
        }
    }
}

fn path_simplify(path: &[Vector], threshold: f64, new_path: &mut NewPath<Vector>) {
    if path.len() < 3 {
        new_path.extend_from_slice(path);
        return;
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
        path_simplify(&path[..=index], threshold, new_path);
        new_path.pop();
        path_simplify(&path[index..], threshold, new_path);
    } else {
        new_path.extend([a, b]);
    }
}

fn path_to_svg(path: &[Vector]) -> String {
    let coords: Vec<String> = path.iter().map(|v| format!("{},{}", v.x, v.y)).collect();
    let points = coords.join(" ");
    format!(
        "<polyline stroke=\"black\" fill=\"none\" points=\"{}\" />",
        points
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Endpoint(usize);

impl Endpoint {
    #[inline]
    fn new(path_id: usize, is_last: bool) -> Self {
        Endpoint(path_id * 2 + if is_last { 1 } else { 0 })
    }
    #[inline]
    fn path_id(self) -> usize {
        self.0 / 2
    }
    #[inline]
    fn is_last(self) -> bool {
        (self.0 & 1) != 0
    }
    #[inline]
    fn opposite(self) -> Self {
        Endpoint(self.0 ^ 1)
    }
}

struct PathGraph {
    links: Vec<usize>,
}

impl PathGraph {
    fn new(path_count: usize) -> Self {
        Self {
            links: vec![usize::MAX; path_count * 2],
        }
    }

    #[inline]
    fn link(&mut self, a: Endpoint, b: Endpoint) {
        self.links[a.0] = b.0;
        self.links[b.0] = a.0;
    }

    #[inline]
    fn is_connected(&self, ep: Endpoint) -> bool {
        self.links[ep.0] != usize::MAX
    }

    #[inline]
    fn next(&self, ep: Endpoint) -> Option<Endpoint> {
        let val = self.links[ep.0];
        if val == usize::MAX {
            None
        } else {
            Some(Endpoint(val))
        }
    }
}
