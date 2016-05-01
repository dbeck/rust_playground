
use std::sync::atomic::{AtomicUsize, Ordering};

struct CircularBuffer<T : Copy> {
  seqno       : AtomicUsize,
  data        : Vec<T>,
  size        : usize,

  // reference numbers to data items (writer, reader and tmp):
  write_to    : Vec<AtomicUsize>,
  read_from   : Vec<usize>,
  write_tmp   : usize,
  max_read    : usize,
}

struct CircularBufferIterator<'a, T: 'a + Copy> {
  data   : &'a [T],
  revpos : &'a [usize],
  count  : usize,
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
      max_read   : 0,
    };

    // make sure there is enough place and fill it with the
    // default value
    ret.data.resize((size*2)+1, default_value);

    for i in 0..size {
      ret.write_to.push(AtomicUsize::new((1+i) << 16));
      ret.read_from.push(1+size+i);
    }

    ret
  }

  fn put<F>(&mut self, setter: F) -> usize
    where F : FnMut(&mut T)
  {
    let mut setter = setter;

    // get a reference to the data
    let mut opt : Option<&mut T> = self.data.get_mut(self.write_tmp);

    // write the data to the temporary writer buffer
    match opt.as_mut() {
      Some(v) => setter(v),
      None    => { panic!("write tmp pos is out of bounds {}", self.write_tmp); }
    }

    // calculate writer flag position
    let seqno  = self.seqno.load(Ordering::SeqCst);
    let pos    = seqno % self.size;

    // get a reference to the writer flag
    match self.write_to.get_mut(pos) {
      Some(v) => {
        let mut old_flag : usize = (*v).load(Ordering::SeqCst);
        let mut old_pos  : usize = old_flag >> 16;
        let new_flag     : usize = (self.write_tmp << 16) + (seqno & 0xffff);

        loop {
          let result = (*v).compare_and_swap(old_flag,
                                             new_flag,
                                             Ordering::SeqCst);
          if result == old_flag {
            self.write_tmp = old_pos;
            break;
          } else {
            old_flag = result;
            old_pos  = old_flag >> 16;
          };
        };
      },
      None => { panic!("write_to index is out of bounds {}", pos); }
    }

    // increase sequence number
    self.seqno.fetch_add(1, Ordering::SeqCst)
  }

  fn iter(&mut self) -> CircularBufferIterator<T> {
    let mut seqno : usize = self.seqno.load(Ordering::SeqCst);
    let mut count : usize = 0;

    loop {
      if count >= self.size || seqno == 0 { break; }
      let pos = (seqno-1) % self.size;

      match self.read_from.get_mut(count) {
        Some(r) => {
          match self.write_to.get_mut(pos) {
            Some(v) => {
              let old_flag : usize = (*v).load(Ordering::SeqCst);
              let old_pos  : usize = old_flag >> 16;
              let old_seq  : usize = old_flag & 0xffff;
              let new_flag : usize = (*r << 16) + (old_seq & 0xffff);
              
              if old_flag == (*v).compare_and_swap(old_flag, new_flag, Ordering::SeqCst) {
                *r = old_pos;
                seqno -=1;
                count += 1;
              } else {
                break;
              }
            },
            None => { panic!("write_to index is out of bounds {}", pos); }
          }
        },
        None => { panic!("read_from index is out of bounds {}", count); }
      }
    }

    CircularBufferIterator {
      data    : self.data.as_slice(),
      revpos  : self.read_from.as_slice(),
      count   : count,
    }
  }
}

impl <'_, T: '_ + Copy> Iterator for CircularBufferIterator<'_, T> {
  type Item = T;

  fn next(&mut self) -> Option<T> {
    if self.count > 0 {
      self.count -= 1;
      let pos : usize = self.revpos[self.count];
      Some(self.data[pos])
    } else {
      None
    }
  }
}

pub fn tests() {
  let mut x = CircularBuffer::new(4, 0 as i32);

  {
    x.put(|v| *v = 1);
    x.put(|v| *v = 2);
    x.put(|v| *v = 3);
    x.put(|v| *v = 4);
    x.put(|v| *v = 5);
  }

  println!("T: {:?}", x.write_tmp);

  for i in &x.write_to {
    let pos = i.load(Ordering::SeqCst) >> 16;
    let seq = i.load(Ordering::SeqCst) & 0xffff;
    println!("W: {:?}/{:?}", pos,seq);
  }

  for i in &x.read_from {
    println!("R: {:?}", i);
  }

  {
    for i in x.iter() {
      println!("--: {}", i);
    }
  }

  println!("T: {:?}", x.write_tmp);

  for i in &x.write_to {
    let pos = i.load(Ordering::SeqCst) >> 16;
    let seq = i.load(Ordering::SeqCst) & 0xffff;
    println!("W: {:?}/{:?}", pos,seq);
  }

  for i in &x.read_from {
    println!("R: {:?}", i);
  }
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
