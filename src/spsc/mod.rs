
#[allow(unused_imports)]
use std::sync::atomic::{AtomicUsize, Ordering};

#[allow(dead_code)]
struct CircularBuffer<T : Copy> {
  seqno      : AtomicUsize,
  data       : Vec<T>,
  size       : usize,

  // reference numbers to data items (writer, reader and tmp):
  write_to   : Vec<AtomicUsize>,
  read_from  : Vec<usize>,
  write_tmp  : usize,
}

impl <T : Copy> CircularBuffer<T> {

  fn new(size : usize, default_value : T) -> CircularBuffer<T> {

    if size == 0 { panic!("size cannot be zero"); }

    let mut ret = CircularBuffer {
      seqno      : AtomicUsize::new(0),
      data       : vec![],
      size       : size,
      write_to   : vec![],
      read_from  : vec![],
      write_tmp  : 0,
    };

    // make sure there is enough place and fill it with the
    // default value
    ret.data.resize((size*2)+1, default_value);

    for i in 0..size {
      ret.write_to.push(AtomicUsize::new(1+i));
      ret.read_from.push(1+size+i);
    }

    ret
  }
}

pub fn tests() {
  let mut _x = CircularBuffer::new(1, 0 as i32);
}

#[cfg(test)]
mod tests {
  //use super::CircularBuffer;

  #[test]
  #[should_panic]
  fn t0() {
    panic!("panic");
  }

  #[test]
  fn t1() {
  }
}
