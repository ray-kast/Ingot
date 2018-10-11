use render::{Pixel, RenderProc, Tile};
use std::{sync::Arc, thread, time::Duration};

pub struct GlitchRenderProc {}

impl GlitchRenderProc {
  pub fn new() -> Self {
    Self {}
  }
}

impl RenderProc for GlitchRenderProc {
  fn process_tile(&self, tile: Arc<Tile>) {
    let mut out_buf = tile.out_buf();

    for i in 0..out_buf.len() {
      out_buf[i] = Pixel::new(0.0, 0.5, 0.0, 1.0);
    }

    thread::sleep(Duration::from_millis(500));
  }
}
