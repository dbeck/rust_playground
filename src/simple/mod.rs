
struct CircularBuffer<T : Copy> {
  seqno : usize,
  data  : Vec<T>,
}

struct CircularBufferIterator<'a, T: 'a + Copy> {
  buffer    : &'a CircularBuffer<T>,
  position  : usize,
  limit     : usize,
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

  fn min_pos(&self) -> usize {
    if self.seqno < self.data.len() {
      0
    } else {
      (self.seqno - self.data.len()) as usize
    }
  }

  fn iter(&self) -> CircularBufferIterator<T> {
    CircularBufferIterator{
      buffer: self,
      position: self.min_pos(),
      limit: self.seqno
     }
  }

  fn put<F>(&mut self, setter: F) -> usize
    where F : FnMut(&mut T)
  {
    // calculate where to put the data
    let pos = self.seqno % self.data.len();

    // get a reference to the data
    let mut opt : Option<&mut T> = self.data.get_mut(pos);

    let mut setter = setter;

    // make sure the index worked
    match opt.as_mut() {
      Some(v) => setter(v),
      None    => { panic!("out of bounds {}", pos); }
    }

    // increase sequence number
    self.seqno += 1;
    self.seqno
  }

  fn get(&self, at: &usize) -> (Option<&T>, usize) {

    let min_pos : usize = if self.seqno < self.data.len() {
      0
    } else {
      (self.seqno - self.data.len()) as usize
    };

    if *at < min_pos      { return (Option::None, min_pos);    }
    if *at >= self.seqno  { return (Option::None, self.seqno); }

    let pos = *at % self.data.len();

    (self.data.get(pos), *at+1)
  }

  fn get_range<F>(&self, from: &usize, to: &usize, getter: F) -> usize
    where F : FnMut(&T)
  {
    if from > to { panic!("from:{} > to:{}", from, to); }

    // figure out the minimum and maximum position
    let min_pos : usize = if self.seqno < self.data.len() {
      0
    } else {
      (self.seqno - self.data.len()) as usize
    };

    let max_pos : usize = self.seqno;

    // cannot return past (overwritten) elements
    if *to < min_pos { return min_pos; }

    // cannot return future elements
    if *from >= max_pos { return max_pos; }

    // constrain start to min_pos
    let start : usize = if *from > min_pos {
      *from
    } else {
      min_pos
    };

    // constrain end to max_pos
    let end : usize = if *to > max_pos {
      max_pos
    } else {
      *to
    };

    let start_pos   = start % self.data.len();
    let end_pos     = end % self.data.len();
    let mut getter  = getter;

    let mut get_range_fn = |from_pos: usize, to_pos: usize| {
      for i in from_pos..to_pos {
        let act_value : Option<&T> = self.data.get(i);
        match act_value {
          Some(v) => { getter(v); }
          None => { panic!("index is out of range i={}",i); }
        };
      }
    };

    if end_pos > start_pos {
      // one contigous range
      get_range_fn(start_pos, end_pos);
    } else {
      // range split into two
      get_range_fn(start_pos, self.data.len());
      get_range_fn(0, end_pos);
    }

    end
  }
}

impl <'_, T: '_ + Copy> Iterator for CircularBufferIterator<'_, T> {
  type Item = T;

  fn next(&mut self) -> Option<T> {
    if self.position >= self.limit {
      None
    } else {
      let at = self.position % self.buffer.data.len();
      match self.buffer.data.get(at) {
        Some(v) => Some(*v),
        None => None
      }
    }
  }
}

pub fn tests() {
  let mut x = CircularBuffer::new(4, 0 as i32);
  x.put(|v| *v = 1);
  let mut y : i32 = 2;
  x.put(|v| { *v = y; y += 1; });
  x.put(|v| { *v = y; y += 1; });
  x.put(|v| { *v = y; y += 1; });
  x.put(|v| { *v = y; y += 1; });

  let ret = x.get_range( &0, &100, |val : &i32| {
    println!("A: {}", *val);
  });
  println!("ret={} min_pos={}", ret,x.min_pos());

  let it = x.iter();
  for i in it {
    println!("B: {}", i);
  }

  let count  = x.iter().count();
  let max    = x.iter().max();
  let first  = x.iter().take(1).last();
  let last   = x.iter().last();

  println!("count={} max={} first={} last={}",
    count,
    max.unwrap(),
    first.unwrap(),
    last.unwrap());
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
  x.put(&my_fn);
  x.put(&my_fn);
  x.put(&my_fn);
}

#[test]
fn can_put_with_env2() {
  let mut x = CircularBuffer::new(1, 0 as i32);
  let mut y = 0;
  x.put(|v| { *v = y; y += 1; });
}
