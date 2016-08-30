use std::collections::VecDeque;
use std::io::{Result as IoResult, Read};
use std::collections::vec_deque::{Drain, Iter};

#[derive(Debug)]
pub struct Grow<R> {
    input: R,
    deq: VecDeque<u8>,
    read_pos: usize
}

impl<R: Read> Grow<R> {
    pub fn new(input: R) -> Grow<R> {
        Grow {
            input: input,
            deq: VecDeque::new(),
            read_pos: 0
        }
    }

    pub fn drain<'a>(&'a mut self, amount: usize) -> Drain<'a, u8> {
        // TODO: add overflow check
        self.read_pos -= amount;
        self.deq.drain(..amount)
    }

    // RangeArgument is not yet stabilised
    // this method should vanish
    pub fn drain_all<'a>(&'a mut self) -> Drain<'a, u8> {
        self.read_pos = 0;
        self.deq.drain(..)
    }

    pub fn iter(&self) -> Iter<u8> {
        self.deq.iter()
    }

    fn copy_to(&mut self, buf: &mut [u8]) -> usize {
        self.deq.iter()
            .skip(self.read_pos)
            .take(buf.len())
            .enumerate()
            .fold(0, |sum, (index, &value)| {
                buf[index] = value;
                sum + 1
            })
    }
}

impl<R: Read> Read for Grow<R> {
    fn read(&mut self, mut buf: &mut [u8]) -> IoResult<usize> {
        let bytes_written = if self.deq.len() - self.read_pos < buf.len() {
            // (ab)use the provided buffer as a read-in cache
            // TODO: only do this if buffer.len() > 1024 to prevent read trashing
            // provide fallback in such case
            let bytes_read = try!(self.input.read(&mut buf)) as usize;
            self.deq.extend(&buf[..bytes_read]);

            if self.read_pos == 0 {
                // by (ab)using the buffer we have just filled the output buffer as well
                // no need to copy over
                bytes_read
            } else {
                self.copy_to(buf)
            }
        } else {
            self.copy_to(buf)
        };

        self.read_pos += bytes_written;
        Ok(bytes_written)
    }
}