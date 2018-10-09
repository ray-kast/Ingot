extern crate gtk;

use gtk::{prelude::*, Button, Window, WindowType};

fn main() {
  gtk::init().unwrap();

  let window = Window::new(WindowType::Toplevel);

  window.set_title("Heyo");
  window.set_default_size(350, 70);

  let button = Button::new_with_label("OK");

  window.add(&button);
  window.show_all();

  window.connect_delete_event(|_, _| {
    gtk::main_quit();
    Inhibit(false)
  });

  button.connect_clicked(|_| {
    println!("fucc");
  });

  gtk::main();
}
