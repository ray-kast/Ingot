use super::prelude::*;
use std::cmp;

struct Data {
  w: u32,
  h: u32,
}

struct Proc {
  data: RwLock<Data>,
  param_radius: Arc<RangedParam<i32>>,
}

pub struct NaiveMedianFilter {
  params: Vec<Param>,
  proc: Arc<Proc>,
}

impl NaiveMedianFilter {
  pub fn new() -> Self {
    let param_radius = Arc::new(RangedParam::new(3, 0, 20, 0, None));

    Self {
      params: vec![Param("Radius".to_string(), param_radius.clone().into())],
      proc: Arc::new(Proc {
        data: RwLock::new(Data { w: 0, h: 0 }),
        param_radius,
      }),
    }
  }
}

impl Filter for NaiveMedianFilter {
  fn name(&self) -> &str { "Median Blur (naive)" }

  fn params(&self) -> &Vec<Param> { &self.params }

  fn proc(&self) -> ArcProc { self.proc.clone() as ArcProc }
}

impl Proc {
  fn process_px<'a>(
    &self,
    tile: &Tile,
    data: &RwLockReadGuard<'a, Data>,
    r: u32,
    c: u32,
    radius: u32,
  ) -> Pixel {
    if radius < 1 {
      return tile.get_input(c, r);
    }

    let mut samples: [Vec<Quantum>; 4] = Default::default();

    let r = r as i32;
    let c = c as i32;
    let radius = radius as i32;

    for r2 in (r - radius)..(r + radius) {
      let r2 =
        cmp::max(0, cmp::min((data.h - 1) as i32, r2 + tile.y() as i32)) as u32;

      for c2 in (c - radius)..(c + radius) {
        let c2 =
          cmp::max(0, cmp::min((data.w - 1) as i32, c2 + tile.x() as i32))
            as u32;

        let px = tile.global_input(c2, r2);

        for i in 0..4 {
          samples[i].push(px[i]);
        }
      }
    }

    for i in 0..4 {
      samples[i].sort_by(|a, b| a.partial_cmp(b).unwrap());
    }

    Pixel::new(
      samples[0][samples[0].len() / 2],
      samples[1][samples[1].len() / 2],
      samples[2][samples[2].len() / 2],
      samples[3][samples[3].len() / 2],
    )
  }
}

impl RenderProc for Proc {
  fn begin(&self, w: u32, h: u32) {
    let mut data = self.data.write().unwrap();

    data.w = w;
    data.h = h;
  }

  fn process_tile(&self, tile: &Tile, cancel_tok: &CancelTok) {
    let data = self.data.read().unwrap();
    let mut out_buf = tile.out_buf();

    let radius = self.param_radius.get() as u32;

    if radius < 30 {
      'row_loop_a: for r in 0..tile.h() {
        let r_stride = r * tile.w();

        if cancel_tok.cancelled() {
          break 'row_loop_a;
        }

        for c in 0..tile.w() {
          out_buf[(r_stride + c) as usize] =
            self.process_px(tile, &data, r, c, radius);
        }
      }
    } else {
      'row_loop_b: for r in 0..tile.h() {
        let r_stride = r * tile.w();

        for c in 0..tile.w() {
          if cancel_tok.cancelled() {
            break 'row_loop_b;
          }

          out_buf[(r_stride + c) as usize] =
            self.process_px(tile, &data, r, c, radius);
        }
      }
    }
  }
}
