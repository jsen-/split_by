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

    fn _next(&mut self) -> Option<Vec<u8>> {
        // TODO: find out why moving the line below here from both branches causes a panic
        // let mut g = self.g.borrow_mut();
        match self.matches.next() {
            None => {
                let mut g = self.g.borrow_mut();
                let rest: Vec<u8> = g.drain_all().collect();

                if rest.len() == 0 {
                    None
                } else {
                    Some(rest)
                }
            },
            Some(m) => {
                let mut g = self.g.borrow_mut();
                let m = m.unwrap();
                let pos = self.pos;
                let found: Vec<u8> = {
                    let i = g.iter().map(|&v| v);
                    i.take(m.start - pos).collect()
                };

                let len = m.end - pos;
                self.pos += len;

                g.drain(len);
                Some(found)
            } 
        }
    }
}

impl<'a, R: Read, A: Automaton<&'a [u8]> > Iterator for SplitByIter<'a, R, A> {
    type Item = Vec<u8>;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self._next() {
                None => return None,
                Some(v) => if v.len() != 0 {
                    return Some(v)
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
/// use std::fs::File;
/// 
/// # fn main() {
/// let ac = AcAutomaton::new(vec!["--------".as_bytes(), "********".as_bytes(), "########".as_bytes()]);
///
/// for split in File::open("path/to/file").unwrap().split_by(&ac) {
///     assert!(split != "--------".as_bytes());
///     assert!(split != "********".as_bytes());
///     assert!(split != "########".as_bytes());
/// }
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
        let ac = AcAutomaton::new(vec!["==".as_bytes()]);
        assert!("==1==2==3==4==5==6==7==8".as_bytes().split_by(&ac).map(|f| f[0]).collect::<Vec<u8>>() == vec![b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8']);
    }
    #[test]
    fn trailing() {
        let ac = AcAutomaton::new(vec!["==".as_bytes()]);
        assert!("1==2==3==4==5==6==7==8==".as_bytes().split_by(&ac).map(|f| f[0]).collect::<Vec<u8>>() == vec![b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8']);
    }
    #[test]
    fn both() {
        let ac = AcAutomaton::new(vec!["==".as_bytes()]);
        assert!("1==2==3==4==5==6==7==8".as_bytes().split_by(&ac).map(|f| f[0]).collect::<Vec<u8>>() == vec![b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8']);
    }
    #[test]
    fn consecutive() {
        let ac = AcAutomaton::new(vec!["==".as_bytes()]);
        assert!("1====2==3==4==5==6==7==8".as_bytes().split_by(&ac).map(|f| f[0]).collect::<Vec<u8>>() == vec![b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8']);
    }
    #[test]
    fn empty() {
        let ac = AcAutomaton::new(vec!["==".as_bytes()]);
        assert!("".as_bytes().split_by(&ac).map(|f| f[0]).collect::<Vec<u8>>() == vec![]);
    }
    #[test]
    fn plain() {
        let ac = AcAutomaton::new(vec!["==".as_bytes()]);
        assert!("==".as_bytes().split_by(&ac).map(|f| f[0]).collect::<Vec<u8>>() == vec![]);
    }
    #[test]
    fn not_present() {
        let ac = AcAutomaton::new(vec!["==".as_bytes()]);
        assert!("12345678".as_bytes().split_by(&ac).map(|f| f[0]).collect::<Vec<u8>>() == vec!["12345678".as_bytes()].iter().map(|f|f[0]).collect::<Vec<u8>>());
    }
}