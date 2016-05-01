
struct CircularBuffer<T : Copy> {
  seqno  : usize,
  data   : Vec<T>,
}

struct CircularBufferIterator<'a, T: 'a + Copy> {
  slice  : &'a [T],
  start  : usize,
  end    : usize,
  pos    : usize,
  wrap   : bool,
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

    let min  = self.min_pos();
    let max  = self.seqno;
    let sz   = self.data.len();

    let min_pos  = min % sz;
    let max_pos  = max % sz;

    if self.seqno == 0 { // no data
      CircularBufferIterator {
        slice  : self.data.as_slice(),
        start  : 0,
        end    : 0,
        pos    : 1,
        wrap   : false,
      }
    }
    else if min_pos < max_pos { // no wrap over
      CircularBufferIterator {
        slice  : self.data.as_slice(),
        start  : min_pos,
        end    : max_pos,
        pos    : min_pos,
        wrap   : false,
      }
    } else {
      CircularBufferIterator {
        slice  : self.data.as_slice(),
        start  : max_pos,
        end    : sz,
        pos    : max_pos,
        wrap   : (max_pos != 0),
      }
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
}

impl <'_, T: '_ + Copy> Iterator for CircularBufferIterator<'_, T> {
  type Item = T;

  fn next(&mut self) -> Option<T> {
    if self.pos >= self.end {
      if self.wrap {
        self.pos    = 1;
        self.end    = self.start;
        self.wrap   = false;
        Some(self.slice[0])
      } else {
        None
      }
    } else {
      let at     = self.pos;
      self.pos  += 1;
      Some(self.slice[at])
    }
  }
}

pub fn tests() {
  let mut x = CircularBuffer::new(2, 0 as i32);
  x.put(|v| *v = 1);
  let mut y : i32 = 2;
  x.put(|v| { *v = y; y += 1; });
  x.put(|v| { *v = y; y += 1; });

  let it = x.iter();
  for i in it {
    println!("CB: {}", i);
  }
}

#[cfg(test)]
mod tests {
  use super::CircularBuffer;

  #[test]
  #[should_panic]
  fn create_zero_sized() {
    let _x = CircularBuffer::new(0, 0 as i32);
  }

  #[test]
  fn empty_buffer() {
    let x = CircularBuffer::new(1, 0 as i32);
    let count = x.iter().count();
    assert_eq!(count, 0);
  }

  #[test]
  fn overload_buffer() {
    let mut x = CircularBuffer::new(2, 0 as i32);
    x.put(|v| *v = 1);
    x.put(|v| *v = 2);
    x.put(|v| *v = 3);
    assert_eq!(x.iter().count(), 2);
    assert_eq!(x.iter().last().unwrap(), 3);
    assert_eq!(x.iter().take(1).last().unwrap(), 2);
  }

  #[test]
  fn sum_available() {
    let mut x = CircularBuffer::new(4, 0 as i32);
    x.put(|v| *v = 2);
    x.put(|v| *v = 4);
    x.put(|v| *v = 6);
    x.put(|v| *v = 8);
    x.put(|v| *v = 10);
    assert_eq!(x.iter().count(), 4);
    let sum = x.iter().take(3).fold(0, |acc, num| acc + num);
    assert_eq!(sum, 18);
  }

  #[test]
  fn can_put_with_env() {
    let mut x = CircularBuffer::new(1, 0 as i32);
    let mut y = 0;
    {
      let my_fn = |v : &mut i32| {
        *v = y;
        y += 1;
      };
      x.put(my_fn);
    }
    {
      // the other way is
      x.put(|v| { *v = y; y += 1; });
      // TODO : check if I need this at all
      //x.put(&my_fn);
    }
  }
}
