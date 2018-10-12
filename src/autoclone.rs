macro_rules! autoclone {
  (@param _) => (_);
  (@param $x:ident) => ($x);

  (move || $body:expr) => (move || $body);
  (move |$($p:tt),+| $body:expr) => (move |$(autoclone!(@param $p),)+| $body);

  ($($n:ident),+ => move || $body:expr) => (
    {
      $(let $n = $n.clone();)+
      move || $body
    }
  );

  ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
    {
      $(let $n = $n.clone();)+
      move |$(autoclone!(@param $p),)+| $body
    }
  );
}