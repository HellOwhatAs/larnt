use crate::mesh::Mesh;
use crate::triangle::Triangle;
use crate::util::parse_floats;
use crate::vector::Vector;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn parse_index(value: &str, length: usize) -> usize {
    let n: i64 = value.parse().unwrap_or(0);
    if n < 0 {
        (length as i64 + n) as usize
    } else {
        n as usize
    }
}

pub fn load_obj(path: &str) -> std::io::Result<Mesh> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    let mut vs: Vec<Vector> = vec![Vector::default()]; // 1-based indexing
    let mut triangles = Vec::new();
    
    for line in reader.lines() {
        let line = line?;
        let fields: Vec<&str> = line.split_whitespace().collect();
        
        if fields.is_empty() {
            continue;
        }
        
        let keyword = fields[0];
        let args = &fields[1..];
        
        match keyword {
            "v" => {
                let args_str: Vec<&str> = args.to_vec();
                let f = parse_floats(&args_str);
                let v = Vector::new(f[0], f[1], f[2]);
                vs.push(v);
            }
            "f" => {
                let fvs: Vec<usize> = args.iter()
                    .map(|arg| {
                        let vertex = arg.split('/').next().unwrap_or("0");
                        parse_index(vertex, vs.len())
                    })
                    .collect();
                
                for i in 1..fvs.len() - 1 {
                    let (i1, i2, i3) = (0, i, i + 1);
                    let t = Triangle::new(vs[fvs[i1]], vs[fvs[i2]], vs[fvs[i3]]);
                    triangles.push(t);
                }
            }
            _ => {}
        }
    }
    
    Ok(Mesh::new(triangles))
}
