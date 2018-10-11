use render::{RenderProc, Tile};
use std::sync::{Arc, RwLock};

struct Data {
  w: u32,
  h: u32,
}

// TODO: make the axis configurable
pub struct FlipRenderProc {
  data: RwLock<Data>,
}

impl FlipRenderProc {
  pub fn new() -> Self {
    Self {
      data: RwLock::new(Data { w: 0, h: 0 }),
    }
  }
}

impl RenderProc for FlipRenderProc {
  fn begin(&self, w: u32, h: u32) {
    let mut data = self.data.write().unwrap();

    data.w = w;
    data.h = h;
  }

  fn process_tile(&self, tile: Arc<Tile>) {
    let data = self.data.read().unwrap();
    let mut out_buf = tile.out_buf();

    let x_axis = data.w - 1;
    let y_axis = data.h - 1;

    for r in 0..tile.h() {
      let r_stride = r * tile.w();

      for c in 0..tile.w() {
        let px =
          tile.global_input(x_axis - (tile.x() + c), y_axis - (tile.y() + r));

        out_buf[(r_stride + c) as usize] = px;
      }
    }
  }
}
