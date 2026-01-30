use ln::{OutlineSphere, Scene, Vector};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

fn normalize(values: &[f64], a: f64, b: f64) -> Vec<f64> {
    let lo = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    values
        .iter()
        .map(|&x| {
            let p = (x - lo) / (hi - lo);
            a + p * (b - a)
        })
        .collect()
}

fn low_pass(values: &[f64], alpha: f64) -> Vec<f64> {
    let mut result = Vec::with_capacity(values.len());
    let mut y = 0.0;
    for &x in values {
        y -= alpha * (y - x);
        result.push(y);
    }
    result
}

fn low_pass_noise(rng: &mut StdRng, n: usize, alpha: f64, iterations: usize) -> Vec<f64> {
    let mut result: Vec<f64> = (0..n).map(|_| rng.gen()).collect();
    for _ in 0..iterations {
        result = low_pass(&result, alpha);
    }
    normalize(&result, -1.0, 1.0)
}

fn main() {
    let mut rng = StdRng::seed_from_u64(1211);
    let eye = Vector::new(8.0, 8.0, 8.0);
    let center = Vector::new(0.0, 0.0, 0.0);
    let up = Vector::new(0.0, 0.0, 1.0);
    
    let mut scene = Scene::new();
    
    for _ in 0..50 {
        let n = 200;
        let xs = low_pass_noise(&mut rng, n, 0.3, 4);
        let ys = low_pass_noise(&mut rng, n, 0.3, 4);
        let zs = low_pass_noise(&mut rng, n, 0.3, 4);
        let ss = low_pass_noise(&mut rng, n, 0.3, 4);
        
        let mut position = Vector::new(0.0, 0.0, 0.0);
        for i in 0..n {
            let sphere = OutlineSphere::new(eye, up, position, 0.1);
            scene.add(sphere);
            let s = (ss[i] + 1.0) / 2.0 * 0.1 + 0.01;
            let v = Vector::new(xs[i], ys[i], zs[i]).normalize().mul_scalar(s);
            position = position.add(v);
        }
    }
    
    let width = 380.0 * 5.0;
    let height = 315.0 * 5.0;
    let fovy = 50.0;
    
    let paths = scene.render(eye, center, up, width, height, fovy, 0.1, 100.0, 0.01);
    paths.write_to_png("out.png", width, height);
}
