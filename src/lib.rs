extern crate aho_corasick;
pub use self::aho_corasick::AcAutomaton; 

use self::aho_corasick::{Automaton, StreamMatches};
use std::io::{Result as IoResult, Read};
use std::clone::Clone;

use std::cell::RefCell;
use std::rc::Rc;

mod grow;
use self::grow::Grow;

#[derive(Debug)]
struct GrowWrap<R>(Rc<RefCell<Grow<R>>>);

impl<R: Read> Read for GrowWrap<R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.0.borrow_mut().read(buf)
    }
}

/// An iterator that splits input by arbitrary number of byte sequences  
#[derive(Debug)]
pub struct SplitByIter<'a, R, A: Automaton<&'a [u8]> + 'a > {
    g: Rc<RefCell<Grow<R>>>,
    pos: usize,
    matches: StreamMatches<'a, GrowWrap<R>, &'a [u8], A>,
}

impl<'a, R: Read, A: Automaton<&'a [u8]> > SplitByIter<'a, R, A> {

    fn _next(&mut self) -> Option<IoResult<Vec<u8> > > {
        // TODO: find out why moving the line below here from both branches causes a panic
        // let mut g = self.g.borrow_mut();
        match self.matches.next() {
            None => {
                let mut g = self.g.borrow_mut();
                let rest: Vec<u8> = g.drain_all().collect();

                if rest.len() == 0 {
                    None
                } else {
                    Some(Ok(rest))
                }
            },
            Some(m) => {
                let mut g = self.g.borrow_mut();
                match m {
                    Ok(m) => { 
                        let pos = self.pos;
                        let found: Vec<u8> = {
                            let i = g.iter().map(|&v| v);
                            i.take(m.start - pos).collect()
                        };

                        let len = m.end - pos;
                        self.pos += len;

                        g.drain(len);
                        Some(Ok(found))
                    },
                    Err(err) => Some(Err(err))
                }
            } 
        }
    }
}

impl<'a, R: Read, A: Automaton<&'a [u8]> > Iterator for SplitByIter<'a, R, A> {
    type Item = IoResult<Vec<u8>>;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self._next() {
                None => return None,
                Some(v) => {
                    match v {
                        Ok(v) => if v.len() != 0 {
                            return Some(Ok(v))
                        },
                        Err(err) => return Some(Err(err))
                    }
                }
            }
        }
    }
}

/// Allows spliting any Read stream by arbitrary number of byte sequences
///
/// # Examples
///
/// ```
/// extern crate split_by;
/// 
/// use split_by::{SplitBy, AcAutomaton};
/// 
/// # fn main() {
/// let ac = AcAutomaton::new(vec!["--------".as_bytes(), "********".as_bytes(), "########".as_bytes()]);
/// let mut splits = br#"first
/// --------
/// second
/// ********########
/// third
/// ################
/// last"#.split_by(&ac);
///
/// assert!(splits.next().unwrap().unwrap().as_slice() == b"first\n");
/// assert!(splits.next().unwrap().unwrap().as_slice() == b"\nsecond\n");
/// assert!(splits.next().unwrap().unwrap().as_slice() == b"\nthird\n");
/// assert!(splits.next().unwrap().unwrap().as_slice() == b"\nlast");
/// assert!(splits.next().is_none());
/// # }
/// ```
///
/// The iterator never produces empty vec, even if the input begins or ends with the splitter
/// or if there are consecutive splitters present
pub trait SplitBy<'a, R: Read> {
    fn split_by<A: Automaton<&'a [u8]> >(self, searcher: &'a A) -> SplitByIter<'a, R, A> where Self: Read;
}

impl<'a, R: Read> SplitBy<'a, R> for R {

    fn split_by<A: Automaton<&'a [u8]> >(self, searcher: &'a A) -> SplitByIter<'a, R, A> where Self: Read {
        
        let ref_g = Rc::new(RefCell::new(Grow::new(self)));
        SplitByIter {
            g: ref_g.clone(),
            pos: 0,
            matches: searcher.stream_find(GrowWrap(ref_g)),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{AcAutomaton, SplitBy};

    #[test]
    fn leading() {
        assert!("==1==2==3==4==5==6==7==8".as_bytes().split_by(&AcAutomaton::new(vec!["==".as_bytes()])).map(|f| f.unwrap()[0]).collect::<Vec<u8>>() == vec![b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8']);
    }
    #[test]
    fn trailing() {
        assert!("1==2==3==4==5==6==7==8==".as_bytes().split_by(&AcAutomaton::new(vec!["==".as_bytes()])).map(|f| f.unwrap()[0]).collect::<Vec<u8>>() == vec![b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8']);
    }
    #[test]
    fn both() {
        assert!("1==2==3==4==5==6==7==8".as_bytes().split_by(&AcAutomaton::new(vec!["==".as_bytes()])).map(|f| f.unwrap()[0]).collect::<Vec<u8>>() == vec![b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8']);
    }
    #[test]
    fn consecutive() {
        assert!("1====2==3==4==5==6==7==8".as_bytes().split_by(&AcAutomaton::new(vec!["==".as_bytes()])).map(|f| f.unwrap()[0]).collect::<Vec<u8>>() == vec![b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8']);
    }
    #[test]
    fn empty() {
        assert!("".as_bytes().split_by(&AcAutomaton::new(vec!["==".as_bytes()])).map(|f| f.unwrap()[0]).collect::<Vec<u8>>() == vec![]);
    }
    #[test]
    fn plain() {
        assert!("==".as_bytes().split_by(&AcAutomaton::new(vec!["==".as_bytes()])).map(|f| f.unwrap()[0]).collect::<Vec<u8>>() == vec![]);
    }
    #[test]
    fn not_present() {
        assert!("12345678".as_bytes().split_by(&AcAutomaton::new(vec!["==".as_bytes()])).map(|f| f.unwrap()[0]).collect::<Vec<u8>>() == vec!["12345678".as_bytes()].iter().map(|f|f[0]).collect::<Vec<u8>>());
    }
}