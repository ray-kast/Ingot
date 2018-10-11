use render::{RenderProc, Tile};
use std::sync::Arc;

#[allow(dead_code)]
pub struct PanicRenderProc;

impl RenderProc for PanicRenderProc {
  fn process_tile(&self, _: Arc<Tile>) {
    panic!("debug panic");
  }
}
