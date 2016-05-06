extern crate rpg;

fn main() {
  use std::thread;
  use rpg::*;

  simple::tests();
  spsc::tests();

  let (mut tx, mut rx) = spsc::channel(7, 0 as i32);
  let t = thread::spawn(move|| {
    for i in 1..1000000 {
      tx.put(|v| *v = i);
    }
  });

  for _k in 1..1000 {
    let mut prev = 0;
    for i in rx.iter() {
      if i < prev { panic!("invalid value read!"); }
      prev = i;
    }
  }

  t.join().unwrap();
}
