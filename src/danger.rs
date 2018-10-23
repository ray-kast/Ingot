use glib::WeakRef;
use gtk::{IsA, Object, ObjectExt};
use std::ops::Deref;

// NB: I'm only doing this because these types are only accessed either directly
//     on the main thread, or inside an idle callback

#[derive(Clone)]
pub struct Danger<T>(T)
where
  T: IsA<Object>;

unsafe impl<T> Send for Danger<T> where T: IsA<Object> {}
unsafe impl<T> Sync for Danger<T> where T: IsA<Object> {}

impl<T> From<T> for Danger<T>
where
  T: IsA<Object>,
{
  fn from(obj: T) -> Self {
    Danger(obj)
  }
}

impl<'a, T> From<&'a T> for Danger<T>
where
  T: IsA<Object> + Clone,
{
  fn from(obj: &'a T) -> Self {
    Danger((*obj).clone())
  }
}

impl<T> Deref for Danger<T>
where
  T: IsA<Object>,
{
  type Target = T;

  fn deref(&self) -> &T {
    &self.0
  }
}

#[derive(Clone)]
pub struct DangerWeak<T>(WeakRef<T>)
where
  T: IsA<Object>;

unsafe impl<T> Send for DangerWeak<T> where T: IsA<Object> {}
unsafe impl<T> Sync for DangerWeak<T> where T: IsA<Object> {}

impl<T> From<T> for DangerWeak<T>
where
  T: IsA<Object> + ObjectExt,
{
  fn from(obj: T) -> Self {
    DangerWeak(obj.downgrade())
  }
}

impl<'a, T> From<&'a T> for DangerWeak<T>
where
  T: IsA<Object> + ObjectExt,
{
  fn from(obj: &'a T) -> Self {
    DangerWeak(obj.downgrade())
  }
}

impl<T> Deref for DangerWeak<T>
where
  T: IsA<Object>,
{
  type Target = WeakRef<T>;

  fn deref(&self) -> &WeakRef<T> {
    &self.0
  }
}
