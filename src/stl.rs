//! STL file loader and saver.
//!
//! This module provides functionality to load and save 3D models in STL format.
//! Both ASCII and binary STL formats are supported.
//!
//! # Example
//!
//! ```no_run
//! use larnt::{load_binary_stl, load_stl, save_binary_stl, Scene, Vector};
//!
//! // Load a binary STL file
//! let mesh = load_binary_stl("model.stl").expect("Failed to load STL");
//!
//! let mut scene = Scene::new();
//! scene.add(mesh);
//! ```

use crate::mesh::Mesh;
use crate::triangle::Triangle;
use crate::util::parse_floats;
use crate::vector::Vector;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

/// Loads a triangle mesh from a binary STL file.
///
/// Binary STL files are more compact than ASCII STL files and load faster.
///
/// # Arguments
///
/// * `path` - Path to the binary STL file
///
/// # Example
///
/// ```no_run
/// use larnt::load_binary_stl;
///
/// let mesh = load_binary_stl("model.stl").expect("Failed to load STL");
/// ```
pub fn load_binary_stl(path: &str) -> std::io::Result<Mesh> {
    println!("Loading STL (Binary): {}", path);
    let mut file = File::open(path)?;

    // Read header
    let mut header = [0u8; 84];
    file.read_exact(&mut header)?;
    let count = u32::from_le_bytes([header[80], header[81], header[82], header[83]]) as usize;

    let mut triangles = Vec::with_capacity(count);

    for _ in 0..count {
        let mut buf = [0u8; 50];
        file.read_exact(&mut buf)?;

        let v1 = Vector::new(
            f32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]) as f64,
            f32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]]) as f64,
            f32::from_le_bytes([buf[20], buf[21], buf[22], buf[23]]) as f64,
        );
        let v2 = Vector::new(
            f32::from_le_bytes([buf[24], buf[25], buf[26], buf[27]]) as f64,
            f32::from_le_bytes([buf[28], buf[29], buf[30], buf[31]]) as f64,
            f32::from_le_bytes([buf[32], buf[33], buf[34], buf[35]]) as f64,
        );
        let v3 = Vector::new(
            f32::from_le_bytes([buf[36], buf[37], buf[38], buf[39]]) as f64,
            f32::from_le_bytes([buf[40], buf[41], buf[42], buf[43]]) as f64,
            f32::from_le_bytes([buf[44], buf[45], buf[46], buf[47]]) as f64,
        );

        let t = Triangle::new(v1, v2, v3);
        triangles.push(t);
    }

    Ok(Mesh::new(triangles))
}

/// Saves a triangle mesh to a binary STL file.
///
/// # Arguments
///
/// * `path` - Path to save the STL file
/// * `mesh` - The mesh to save
///
/// # Example
///
/// ```no_run
/// use larnt::{save_binary_stl, load_obj};
///
/// let mesh = load_obj("model.obj").expect("Failed to load OBJ");
/// save_binary_stl("output.stl", &mesh).expect("Failed to save STL");
/// ```
pub fn save_binary_stl(path: &str, mesh: &Mesh) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // Write header
    let header = [0u8; 80];
    writer.write_all(&header)?;

    // Write count
    let count = mesh.triangles.len() as u32;
    writer.write_all(&count.to_le_bytes())?;

    // Write triangles
    for triangle in &mesh.triangles {
        // Normal (placeholder - zeros)
        writer.write_all(&[0u8; 12])?;

        // V1
        writer.write_all(&(triangle.v1.x as f32).to_le_bytes())?;
        writer.write_all(&(triangle.v1.y as f32).to_le_bytes())?;
        writer.write_all(&(triangle.v1.z as f32).to_le_bytes())?;

        // V2
        writer.write_all(&(triangle.v2.x as f32).to_le_bytes())?;
        writer.write_all(&(triangle.v2.y as f32).to_le_bytes())?;
        writer.write_all(&(triangle.v2.z as f32).to_le_bytes())?;

        // V3
        writer.write_all(&(triangle.v3.x as f32).to_le_bytes())?;
        writer.write_all(&(triangle.v3.y as f32).to_le_bytes())?;
        writer.write_all(&(triangle.v3.z as f32).to_le_bytes())?;

        // Attribute byte count
        writer.write_all(&[0u8; 2])?;
    }

    Ok(())
}

/// Loads a triangle mesh from an ASCII STL file.
///
/// ASCII STL files are human-readable but larger than binary STL files.
///
/// # Arguments
///
/// * `path` - Path to the ASCII STL file
///
/// # Example
///
/// ```no_run
/// use larnt::load_stl;
///
/// let mesh = load_stl("model.stl").expect("Failed to load STL");
/// ```
pub fn load_stl(path: &str) -> std::io::Result<Mesh> {
    println!("Loading STL (ASCII): {}", path);
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut vertices = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let fields: Vec<&str> = line.split_whitespace().collect();

        if fields.len() == 4 && fields[0] == "vertex" {
            let f = parse_floats(&fields[1..]);
            vertices.push(Vector::new(f[0], f[1], f[2]));
        }
    }

    let mut triangles = Vec::new();
    for i in (0..vertices.len()).step_by(3) {
        if i + 2 < vertices.len() {
            let t = Triangle::new(vertices[i], vertices[i + 1], vertices[i + 2]);
            triangles.push(t);
        }
    }

    Ok(Mesh::new(triangles))
}
