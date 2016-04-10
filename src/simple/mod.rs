
struct CircularBuffer<T : Copy> {
  seqno : usize,
  data  : Vec<T>,
}

impl <T : Copy> CircularBuffer<T> {
  fn new(size : usize, default_value : T) -> CircularBuffer<T> {

    if size == 0 { panic!("size cannot be zero"); }

    let mut ret = CircularBuffer {
      seqno : 0,
      data  : vec![],
    };

    // make sure there is enough place and fill it with the
    // default value
    ret.data.resize(size, default_value);
    ret
  }

  fn put<F>(&mut self, setter: F) -> usize
    where F : Fn(&mut T)
  {
    // calculate where to put the data
    let pos = self.seqno % self.data.len();

    // get a reference to the data
    let mut opt : Option<&mut T> = self.data.get_mut(pos);

    // make sure the index worked
    match opt.as_mut() {
      Some(v) => setter(v),
      None    => { panic!("out of bounds {}", pos); }
    }

    // increase sequence number
    self.seqno += 1;
    self.seqno
  }

  fn put_mut<F>(&mut self, mut setter: F) -> usize
    where F : FnMut(&mut T)
  {
    // calculate where to put the data
    let pos = self.seqno % self.data.len();

    // get a reference to the data
    let mut opt : Option<&mut T> = self.data.get_mut(pos);

    // make sure the index worked
    match opt.as_mut() {
      Some(v) => setter(v),
      None    => { panic!("out of bounds {}", pos); }
    }

    // increase sequence number
    self.seqno += 1;
    self.seqno
  }
}

pub fn tests() {
  let mut x = CircularBuffer::new(4, 0 as i32);
  x.put(|v| *v = 1);
  let mut y = 0;
  x.put_mut(|v| { *v = y; y += 1; });
}

#[test]
#[should_panic]
fn create_zero_sized() {
  let _x = CircularBuffer::new(0, 0 as i32);
}

#[test]
fn create_non_zero_sized() {
  let _x = CircularBuffer::new(1, 0 as i32);
}

#[test]
fn can_put() {
  let mut x = CircularBuffer::new(1, 0 as i32);
  x.put(|v| *v = 1);
}

#[test]
fn can_put_with_env() {
  let mut x = CircularBuffer::new(1, 0 as i32);
  let mut y = 0;
  let my_fn = |v : &mut i32| {
    *v = y;
    y += 1;
  };
  x.put_mut(my_fn);
}

#[test]
fn can_put_with_env2() {
  let mut x = CircularBuffer::new(1, 0 as i32);
  let mut y = 0;
  x.put_mut(|v| { *v = y; y += 1; });
}
