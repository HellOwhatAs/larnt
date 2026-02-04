pub struct BilinearGrid {
    width: usize,
    height: usize,
    data: Vec<f64>,

    x_origin: f64,
    y_origin: f64,
    x_scale: f64,
    y_scale: f64,
}

impl BilinearGrid {
    pub fn new(
        width: usize, 
        height: usize, 
        data: Vec<f64>, 
        x_range: (f64, f64), 
        y_range: (f64, f64)
    ) -> Self {
        let x_scale = (width - 1) as f64 / (x_range.1 - x_range.0);
        let y_scale = (height - 1) as f64 / (y_range.1 - y_range.0);

        Self {
            width,
            height,
            data,
            x_origin: x_range.0,
            y_origin: y_range.0,
            x_scale,
            y_scale,
        }
    }

    #[inline(always)]
    fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a + (b - a) * t
    }

    pub fn get(&self, real_x: f64, real_y: f64) -> f64 {
        let u = (real_x - self.x_origin) * self.x_scale;
        let v = (real_y - self.y_origin) * self.y_scale;

        let u = u.clamp(0.0, (self.width - 1) as f64);
        let v = v.clamp(0.0, (self.height - 1) as f64);

        let ix = u.floor() as usize;
        let iy = v.floor() as usize;
        
        let tx = u - ix as f64;
        let ty = v - iy as f64;

        let ix0 = ix;
        let iy0 = iy;
        let ix1 = (ix + 1).min(self.width - 1);
        let iy1 = (iy + 1).min(self.height - 1);

        let q00 = self.data[iy0 * self.width + ix0];
        let q10 = self.data[iy0 * self.width + ix1];
        let q01 = self.data[iy1 * self.width + ix0];
        let q11 = self.data[iy1 * self.width + ix1];

        let top = Self::lerp(q00, q10, tx);
        let bot = Self::lerp(q01, q11, tx);

        Self::lerp(top, bot, ty)
    }
}
