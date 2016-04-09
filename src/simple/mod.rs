

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

    // hand over the object
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
    self.seqno = self.seqno + 1;
    self.seqno
  }

  fn get<F>(&self, from: &usize, to: &usize, getter: F) -> usize
    where F : Fn(&T)
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

    let start_pos = start % self.data.len();
    let end_pos   = end % self.data.len();

    let get_range = |from_pos: usize, to_pos: usize| {
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
      get_range(start_pos, end_pos);
    } else {
      // range split into two
      get_range(start_pos, self.data.len());
      get_range(0, end_pos);
    }

    end
  }
}

pub fn tests() {
  let mut x = CircularBuffer::new(4, 0 as i32);
  x.put(|v| *v = 1);
  x.put(|v| *v = 2);
  x.put(|v| *v = 3);

  let ret = x.get( &0, &100, |val : &i32| {
    println!("A: {}",*val);
  });

  println!("A: ret={} {} {}",ret,x.seqno,x.data.len());

  x.put(|v| *v = 4);
  x.put(|v| *v = 5);
  x.put(|v| *v = 6);

  let ret2 = x.get( &ret, &100, |val : &i32| {
    println!("B: {}",*val);
  });

  println!("B: ret2={} {} {}",ret2,x.seqno,x.data.len());
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
