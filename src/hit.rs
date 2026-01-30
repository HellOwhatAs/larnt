use crate::common::INF;

#[derive(Debug, Clone, Copy)]
pub struct Hit {
    pub t: f64,
    pub ok: bool,
}

impl Hit {
    pub fn new(t: f64) -> Self {
        Hit { t, ok: true }
    }

    pub fn no_hit() -> Self {
        Hit { t: INF, ok: false }
    }

    pub fn is_ok(&self) -> bool {
        self.t < INF
    }

    pub fn min(&self, other: Hit) -> Hit {
        if self.t <= other.t {
            *self
        } else {
            other
        }
    }

    pub fn max(&self, other: Hit) -> Hit {
        if self.t > other.t {
            *self
        } else {
            other
        }
    }
}

impl Default for Hit {
    fn default() -> Self {
        Hit::no_hit()
    }
}
