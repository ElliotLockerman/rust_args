
use std::collections::{BTreeMap, HashSet};
use std::str::FromStr;
use std::fmt::Debug;
use std::cell::{RefCell};


pub trait PosArgBase {
    fn name(&self) -> &str;
    fn desc(&self) -> &str;
    fn found(&self) -> bool;
    fn parse(&mut self, s: &str);
}

pub struct PosArg<T> where T: FromStr, <T as FromStr>::Err: Debug {
    name: String,
    desc: String,
    val: Option<T>,
}

impl<T> PosArg<T> 
    where T: FromStr, 
        <T as FromStr>::Err: Debug {

    pub fn new(name: String, desc: String) -> Self {
        Self{name, desc, val: None}
    }

    pub fn val(&mut self) -> Option<T> { self.val.take() }
}

impl<T> PosArgBase for PosArg<T> 
    where T: FromStr, 
        <T as FromStr>::Err: Debug {

    fn name(&self) -> &str { &self.name }
    fn desc(&self) -> &str { &self.desc }
    fn found(&self) -> bool { self.val.is_some() }

    fn parse(&mut self, s: &str) {
        self.val = T::from_str(s).ok();
    }
}






pub trait KVArgBase {
    fn name(&self) -> &str;
    fn desc(&self) -> &str;
    fn short_key(&self) -> Option<char>; // Not valid for positional argument
    fn found(&self) -> bool;

    fn parse(&mut self, s: &str);
}

pub struct KVArg<T> 
    where T: FromStr, 
        <T as FromStr>::Err: Debug {
    name: String,
    desc: String,
    short_key: Option<char>,
    val: Option<T>,
}

impl<T> KVArg<T> 
    where T: FromStr, 
        <T as FromStr>::Err: Debug {

    pub fn new(name: String, short_key: Option<char>, desc: String) -> RefCell<Self> {
        RefCell::new(Self{name,  desc, val: None, short_key})
    }

    pub fn val(&mut self) -> Option<T> { self.val.take() }
}


impl<T> KVArgBase for KVArg<T> where T: FromStr, <T as FromStr>::Err: Debug {
    fn name(&self) -> &str { &self.name }
    fn desc(&self) -> &str { &self.desc }
    fn short_key(&self) -> Option<char> { self.short_key }
    fn found(&self) -> bool { self.val.is_some() }

    fn parse(&mut self, s: &str) {
        self.val = T::from_str(s).ok();
    }

}





pub trait FlagArgBase {
    fn name(&self) -> &str;
    fn desc(&self) -> &str;
    fn short_key(&self) -> Option<char>; // Not valid for positional argument
    fn found(&self) -> bool;

    fn parse(&mut self);
}


pub struct FlagArg {
    name: String,
    desc: String,
    short_key: Option<char>,
    val: bool,
}

impl FlagArg {
    pub fn new(name: String, desc: String, short_key: Option<char>) -> RefCell<Self> {
        RefCell::new(Self{name,  desc, short_key, val: false})
    }
}

impl FlagArgBase for FlagArg {
    fn name(&self) -> &str { &self.name }
    fn desc(&self) -> &str { &self.desc }
    fn short_key(&self) -> Option<char> { self.short_key }
    fn found(&self) -> bool { self.val }

    fn parse(&mut self) { self.val = true; }
}





pub struct Parser<'a> {
    pos_args: Vec<&'a mut PosArgBase>,
    pos_arg_names: HashSet<String>,

    kv_keys: BTreeMap<String, &'a RefCell<KVArgBase>>,

    flag_keys: BTreeMap<String, &'a RefCell<FlagArgBase>>,
}

impl<'a> Parser<'a> {
    pub fn new() -> Self { 
        Self{
            pos_args: Vec::new(),
            pos_arg_names: HashSet::new(),
            kv_keys: BTreeMap::new(),
            flag_keys: BTreeMap::new(),
        } 
    }


    pub fn add_pos_arg(&mut self, pos_arg: &'a mut PosArgBase) {
        assert!(!self.pos_arg_names.contains(pos_arg.name()));
        self.pos_arg_names.insert(String::from(pos_arg.name()));
        self.pos_args.push(pos_arg);
    }

    pub fn add_kv_arg(&mut self, kv_arg: &'a RefCell<KVArgBase>) {

        assert!(!self.kv_keys.contains_key(kv_arg.borrow().name())
            && !self.flag_keys.contains_key(kv_arg.borrow().name()));
        assert!(kv_arg.borrow().name().len() > 1);

        self.kv_keys.insert(String::from(kv_arg.borrow().name()), kv_arg);

        let short_key = kv_arg.borrow().short_key();
        if let Some(c) = short_key {
            let cs = c.to_string();
            assert!(!self.kv_keys.contains_key(&cs) 
                && !self.flag_keys.contains_key(&cs));
            self.kv_keys.insert(cs, kv_arg);
        };
    }

    pub fn add_flag_arg(&mut self, flag_arg: &'a RefCell<FlagArgBase>) {
        assert!(!self.flag_keys.contains_key(flag_arg.borrow().name())
            && !self.kv_keys.contains_key(flag_arg.borrow().name()));
        assert!(flag_arg.borrow().name().len() > 1);

        self.flag_keys.insert(String::from(flag_arg.borrow().name()), flag_arg);

        let short_key = flag_arg.borrow().short_key();
        if let Some(c) = short_key {
            let cs = c.to_string();
            assert!(!self.flag_keys.contains_key(&cs)
                && !self.kv_keys.contains_key(&cs));
            self.flag_keys.insert(cs, flag_arg);
        };
    }

    pub fn parse(&mut self) {
        self.parse_vec(std::env::args().collect());
    }

    pub fn parse_vec(&mut self, argv: Vec<String>) {

        let mut pos_args_consumed = 0;

        let mut it = argv.iter();
        it.next(); // skip first arg (program path)
        while let Some(arg) = it.next() {
            let mut chars = arg.chars();
            let first = chars.next().unwrap();
            let second = chars.next().unwrap();
            
            if first == '-' {
                // Long key kv arg
                let key = if second == '-' {
                    String::from(&arg[2..])
                } else {
                    String::from(&arg[1..])
                };

                if let Some(arg_rc) = self.kv_keys.get(&key) {
                    assert!(!arg_rc.borrow().found());
                    // let mut q:() = arg_rc;
                    arg_rc.borrow_mut().parse(it.next().unwrap());
                } else if let Some(arg_rc) = self.flag_keys.get(&key) {
                    assert!(!arg_rc.borrow().found());
                    arg_rc.borrow_mut().parse();
                } else {
                    panic!("No such key `{:?}`", key);
                }

            } else {
                // Positional arg
                if pos_args_consumed >= self.pos_args.len() {
                    panic!("Too many positional args!");
                }

                self.pos_args[pos_args_consumed].parse(arg);
                pos_args_consumed += 1;
            }
        }
    }



}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let kv = KVArg::<i32>::new("first".to_string(), Some('f'), "first argument".to_string());
        let mut parser = Parser::new();
        parser.add_kv_arg(&kv);

        let args = vec!["".to_string(), "-f".to_string(), "42".to_string()];

        parser.parse_vec(args);

        assert!(kv.borrow_mut().val().unwrap() == 42);
    }
}
