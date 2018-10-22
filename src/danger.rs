use glib::WeakRef;
use gtk::{IsA, Object, ObjectExt};
use std::ops::Deref;

#[derive(Clone)]
pub struct Danger<T>(WeakRef<T>)
where
  T: IsA<Object>;

unsafe impl<T> Send for Danger<T> where T: IsA<Object> {}
unsafe impl<T> Sync for Danger<T> where T: IsA<Object> {}

impl<T> From<T> for Danger<T>
where
  T: IsA<Object>,
  T: ObjectExt,
{
  fn from(obj: T) -> Self {
    Danger(obj.downgrade())
  }
}

impl<'a, T> From<&'a T> for Danger<T>
where
  T: IsA<Object>,
  T: ObjectExt,
{
  fn from(obj: &'a T) -> Self {
    Danger(obj.downgrade())
  }
}

impl<T> Deref for Danger<T>
where
  T: IsA<Object>,
{
  type Target = WeakRef<T>;

  fn deref(&self) -> &WeakRef<T> {
    &self.0
  }
}
