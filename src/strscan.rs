use regex::{Regex, Captures};
use std::cell::{Cell, RefCell};
use std::rc::{Rc};
use std::{fmt};

#[derive(Debug)]
pub struct StringScanner<'t> {
    string: &'t str,
    pos: Cell<usize>, // current byte index into `string`
    end: usize, // amount of bytes in `string`
    last_match: RefCell<LastMatch<'t>>, // structure containing last match, if any
}

struct LastMatch<'t> {
    caps: Option<Rc<Captures<'t>>>,
}

impl<'t> LastMatch<'t> {
    fn set(&mut self, caps: Option<Rc<Captures<'t>>>) {
        self.caps = caps;
    }
}

impl<'t> fmt::Debug for LastMatch<'t> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.caps {
            None => write!(f, "matchdata: None"),
            Some(ref caps) => {
                write!(f, "matchdata: {:?}", caps.iter_pos().collect::<Vec<_>>())
            }
        }
    }
}

impl<'t> StringScanner<'t> {
    pub fn new<'a>(string: &'a str) -> StringScanner<'a> {
        let last_match = RefCell::new(LastMatch {
            caps: None,
        });
        StringScanner {
            string: string,
            pos: Cell::new(0),
            end: string.len(),
            last_match: last_match,
        }
    }

    // Are we at the beginning of a line?
    pub fn is_bol(&self) -> bool {
        if self.pos.get() == 0 { return true; }
        if self.pos.get() > self.end { return false; }
        match &self.string[self.pos.get() - 1..self.pos.get()] {
            c if c == "\n" => true,
            _ => false
        }
    }

    // Are we at the end of the (entire) string?
    pub fn is_eos(&self) -> bool {
        self.pos.get() == self.end
    }

    pub fn get_pos(&self) -> usize {
        self.pos.get()
    }

    pub fn set_pos(&self, pos: usize) -> bool {
        if pos > self.end {
            return false; // FIXME: return error
        }
        self.pos.set(pos);
        true
    }

    pub fn terminate(&self) {
        self.pos.set(self.end);
    }

    pub fn peek_bytes(&self, len: usize) -> Option<&str> {
        if self.is_eos() { return None; }
        let mut end = self.pos.get() + len;
        if end > self.end {
            end = self.end;
        }
        Some(&self.rest().unwrap()[self.pos.get()..end])
    }

    pub fn peek_chars(&self, mut len: usize) -> Option<&str> {
        if self.is_eos() { return None; }
        let rest = self.rest().unwrap();
        let num_chars_rem = rest.chars().count();
        if len > num_chars_rem {
            len = num_chars_rem
        }
        Some(rest.slice_chars(0, len))
    }

    pub fn rest(&self) -> Option<&str> {
        if self.is_eos() { return None; }
        Some(&self.string[self.pos.get()..])
    }

    pub fn get_byte(&self) -> Option<u8> {
        if self.is_eos() { return None; }
        let byte_slice = &self.rest().unwrap()[self.pos.get()..self.pos.get() + 1];
        self.pos.set(self.pos.get() + 1);
        Some(byte_slice.as_bytes()[0])
    }

    pub fn get_char(&self) -> Option<&str> {
        if self.is_eos() { return None; }
        let rest = &self.string[self.pos.get()..];
        let chr = rest.slice_chars(0, 1);
        self.pos.set(self.pos.get() + chr.len());
        Some(chr)
    }

    pub fn scan(&self, re: &Regex) -> Option<&str> {
        let rest = &self.string[self.pos.get()..];
        let caps_opt = re.captures(rest);
        if caps_opt.is_none() {
            self.last_match.borrow_mut().set(None);
            return None;
        }
        let caps = caps_opt.unwrap();
        match caps.pos(0) {
            Some((_, end_idx)) => {
                let new_pos = self.pos.get() + end_idx;
                let ret = &self.string[self.pos.get()..new_pos];
                self.pos.set(new_pos);
                self.last_match.borrow_mut().set(Some(Rc::new(caps)));
                Some(ret)
            },
            None => unreachable!()
        }
    }

    pub fn check(&self, re: &Regex) -> bool {
        let caps = re.captures(&self.string[self.pos.get()..]);
        match caps {
            Some(cs) => {
                self.last_match.borrow_mut().set(Some(Rc::new(cs)));
                true
            }
            None => {
                self.last_match.borrow_mut().set(None);
                false
            }
        }
    }

    // return captures from last match, if any
    pub fn captures(&self) -> Option<Rc<Captures<'t>>> {
        self.last_match.borrow().caps.clone()
    }

    // return last captured match at position `i`, if any
    pub fn match_at(&self, i: usize) -> Option<&str> {
        match self.captures() {
            Some(caps) => caps.at(i),
            None => None
        }
    }

    // return last captured match with name `name`, if any
    pub fn match_name(&self, name: &str) -> Option<&str> {
        match self.captures() {
            Some(caps) => caps.name(name),
            None => None
        }
    }
}

#[test]
fn test_is_bol() {
    let scanner = StringScanner::new("test\n bol");
    assert!(scanner.is_bol());
    scanner.set_pos(5);
    assert!(scanner.is_bol());
    scanner.set_pos(6);
    assert!(! scanner.is_bol());
}

#[test]
fn test_peek_bytes() {
    let scanner = StringScanner::new("test\n peek");
    let slice = scanner.peek_bytes(1).unwrap();
    assert_eq!("t", slice);
    assert_eq!(0, scanner.get_pos());
    assert_eq!("test\n peek", scanner.peek_bytes(100).unwrap());
    scanner.terminate();
    assert_eq!(None, scanner.peek_bytes(1));
}

#[test]
fn test_peek_chars() {
    let scanner = StringScanner::new("Löwe 老虎 Léopard");
    let slice = scanner.peek_chars(4).unwrap();
    assert_eq!("Löwe", slice);
    assert_eq!(0, scanner.get_pos());
    assert_eq!("Löwe 老虎 Léopard", scanner.peek_chars(100).unwrap());
    scanner.terminate();
    assert_eq!(None, scanner.peek_chars(1));
}

#[test]
fn test_set_pos() {
    let scanner = StringScanner::new("test\n set pos");
    assert_eq!(0, scanner.get_pos());
    assert!(scanner.set_pos(1));
    assert_eq!(1, scanner.get_pos());
    assert!(! scanner.set_pos(100));
    assert_eq!(1, scanner.get_pos());
}

#[test]
fn test_empty_scanner() {
    let scanner = StringScanner::new("");
    assert!(scanner.is_bol());
    assert!(scanner.is_eos());
    scanner.set_pos(1);
    assert!(scanner.is_eos());
}

#[test]
fn test_rest() {
    let scanner = StringScanner::new("test rest");
    assert_eq!("test rest", scanner.rest().unwrap());
    assert!(scanner.is_bol());
    scanner.set_pos(5);
    assert_eq!("rest", scanner.rest().unwrap());
    scanner.terminate();
    assert_eq!(None, scanner.rest());
}

#[test]
fn test_scan() {
    let scanner = StringScanner::new("test\n scan");
    let re_chars = Regex::new(r"^\w+").unwrap();
    let re_ws = Regex::new(r"^\s+").unwrap();
    let scanned = scanner.scan(&re_chars).unwrap();
    assert_eq!("test", scanned);
    assert!(scanner.scan(&re_chars).is_none());
    assert_eq!("\n ", scanner.scan(&re_ws).unwrap());
    assert_eq!("scan", scanner.scan(&re_chars).unwrap());
    assert!(scanner.scan(&re_ws).is_none());
    assert!(scanner.is_eos());
}

#[test]
fn test_check() {
    let scanner = StringScanner::new("test\n check");
    let re_chars = Regex::new(r"^\w+").unwrap();
    let re_ws = Regex::new(r"^\s+").unwrap();
    assert!(scanner.check(&re_chars));
    assert!(! scanner.check(&re_ws));
}

#[test]
fn test_captures() {
    let scanner = StringScanner::new("test\n caps");
    let re = Regex::new(r"^(\w+)\s+").unwrap();
    scanner.check(&re);
    assert_eq!("test\n ", scanner.captures().unwrap().at(0).unwrap());
    assert_eq!("test", scanner.captures().unwrap().at(1).unwrap());
    assert_eq!(None, scanner.captures().unwrap().at(2));
    assert_eq!("test\n ", scanner.match_at(0).unwrap());
    assert_eq!("test", scanner.match_at(1).unwrap());
    assert_eq!(None, scanner.match_at(2));
}
