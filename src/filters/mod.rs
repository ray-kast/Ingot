mod blank;
mod dummy;
mod flip;
mod glitch;
mod invert;
mod naive_median;
mod panic;
pub mod params;

pub use self::{
  blank::*, dummy::*, flip::*, glitch::*, invert::*, naive_median::*, panic::*,
};

// TODO: look into creating a macro to define filters

mod prelude {
  pub use super::{params::*, ArcProc, Filter};
  pub use render::{CancelTok, Pixel, Quantum, RenderProc, Tile};
  pub use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
}

use self::prelude::*;

pub type ArcProc = Arc<RenderProc + Send + Sync>;

pub trait Filter {
  fn name(&self) -> &str;

  fn params(&self) -> &Vec<Param>;

  // TODO: can this be re-structured to avoid all the 'as ArcProc' casting?
  fn proc(&self) -> ArcProc;
}
