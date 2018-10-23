use std::sync::{
  atomic::{AtomicBool, AtomicI32, Ordering},
  Arc, RwLock, RwLockWriteGuard,
};

pub struct Param(pub String, pub ParamVal);

pub enum ParamVal {
  Switch(Arc<BoolParam>),
  SpinInt(Arc<IntParam>),
  RangedInt(Arc<RangedParam<i32>>),
  RangedFloat(Arc<RangedParam<f64>>),
}

use self::ParamVal::*;

impl From<Arc<BoolParam>> for ParamVal {
  fn from(val: Arc<BoolParam>) -> Self { Switch(val) }
}

impl From<Arc<IntParam>> for ParamVal {
  fn from(val: Arc<IntParam>) -> Self { SpinInt(val) }
}

impl From<Arc<RangedParam<i32>>> for ParamVal {
  fn from(val: Arc<RangedParam<i32>>) -> Self { RangedInt(val) }
}

impl From<Arc<RangedParam<f64>>> for ParamVal {
  fn from(val: Arc<RangedParam<f64>>) -> Self { RangedFloat(val) }
}

pub struct BoolParam {
  value: AtomicBool,
}

impl BoolParam {
  pub fn new(default: bool) -> Self {
    Self {
      value: AtomicBool::new(default),
    }
  }

  pub fn get(&self) -> bool { self.value.load(Ordering::SeqCst) }

  pub fn set(&self, val: bool) { self.value.store(val, Ordering::SeqCst); }
}

pub struct IntParam {
  value: AtomicI32,
}

impl IntParam {
  pub fn new(default: i32) -> Self {
    Self {
      value: AtomicI32::new(default),
    }
  }

  pub fn get(&self) -> i32 { self.value.load(Ordering::SeqCst) }

  pub fn set(&self, val: i32) { self.value.store(val, Ordering::SeqCst); }

  pub fn swap(&self, val: i32) -> i32 { self.value.swap(val, Ordering::SeqCst) }
}

struct RangedParamValue<T> {
  internal: T,
  coerced: T,
}

pub struct RangedParam<T>
where
  T: PartialOrd + Copy,
{
  min: T,
  max: T,
  hard_min: Option<T>,
  hard_max: Option<T>,
  value: RwLock<RangedParamValue<T>>,
}

impl<T> RangedParam<T>
where
  T: PartialOrd + Copy,
{
  pub fn new<IN, IX>(
    default: T,
    min: T,
    max: T,
    hard_min: IN,
    hard_max: IX,
  ) -> Self
  where
    Option<T>: From<IN>,
    Option<T>: From<IX>,
  {
    let ret = Self {
      min,
      max,
      hard_min: hard_min.into(),
      hard_max: hard_max.into(),
      value: RwLock::new(RangedParamValue {
        internal: default,
        coerced: default,
      }),
    };

    if let Some(min) = ret.hard_min {
      if let Some(max) = ret.hard_max {
        if max < min {
          panic!("invalid hard limits (max < min)");
        }
      }
    }

    ret.coerce(&mut ret.value.write().unwrap());

    ret
  }

  fn coerce<'a>(&self, value: &mut RwLockWriteGuard<'a, RangedParamValue<T>>) {
    if let Some(min) = self.hard_min {
      if value.internal < min {
        value.coerced = min;
        return;
      }
    }

    if let Some(max) = self.hard_max {
      if value.internal > max {
        value.coerced = max;
        return;
      }
    }

    value.coerced = value.internal;
  }

  pub fn min(&self) -> T { self.min }

  pub fn max(&self) -> T { self.max }

  pub fn get(&self) -> T { self.value.read().unwrap().coerced }

  pub fn set(&self, val: T) {
    let mut value = self.value.write().unwrap();

    value.internal = val;

    self.coerce(&mut value);
  }

  pub fn swap(&self, val: T) -> T {
    let mut value = self.value.write().unwrap();

    let prev = value.coerced;

    value.internal = val;

    self.coerce(&mut value);

    prev
  }
}
