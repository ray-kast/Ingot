mod dummy;
mod flip;
mod glitch;
mod invert;
mod panic;
pub mod params;

pub use self::{dummy::*, flip::*, glitch::*, invert::*, panic::*};

// TODO: look into creating a macro to define filters

mod prelude {
  pub use super::{params::*, ArcProc, Filter};
  pub use render::{CancelTok, Pixel, RenderProc, Tile};
  // TODO: it may be useful to include RwLock
  pub use std::sync::Arc;
}

use self::prelude::*;

pub type ArcProc = Arc<RenderProc + Send + Sync>;

pub trait Filter {
  fn name(&self) -> &str;

  fn params(&self) -> &Vec<Param>;

  // TODO: can this be re-structured to avoid all the 'as ArcProc' casting?
  fn proc(&self) -> ArcProc;
}
