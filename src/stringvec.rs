use ouroboros::self_referencing;
use std::{error::Error, fs, ops::Index, path::{Path, PathBuf}, str::Split};
use std::iter::FromIterator;

const WORDS_PATH: &str = "./words_1000.txt";
pub fn load_words() -> String
{
    return fs::read_to_string(WORDS_PATH).expect(&format!("File not found: {}", WORDS_PATH));
}

#[self_referencing]
#[derive(Debug)]
struct IWordVec
{
    inner: String,
    #[borrows(inner)]
    #[covariant]
    refs:  Vec<&'this str>,
}

impl IWordVec
{
    ///doublequotes in cells dont work.
    fn len(&self)-> usize 
    {
        return self.borrow_refs().len()
    }

    #[rustfmt::skip]
    fn from_csv(src: String) -> IWordVec
    {
        let inner = src;//fs::read_to_string(src)?;
        fn refs_builder<'a>(inner: &'a String) -> Vec<&'a str>
        {
            let mut refs = vec![];

            let qsplit = inner.split('"').array_chunks::<2>();
            let rem = qsplit.clone().into_remainder();
            qsplit.for_each(|[unq,q]|{
                unq.split(&[',', '\t','\n'])
                   .map(|s| s.trim())
                   .filter(|s| s.len()!=0)
                   .for_each(|s| refs.push(s));
                refs.push(q);
            });
            //last term will never be quoted due to how split works. 
            if let Some(rem) = rem
            {
                rem.for_each(|s|{ s
                   .split(&[',', '\t','\n'])
                   .map(|s| s.trim())
                   .filter(|s| s.len()!=0)
                   .for_each(|s| refs.push(s))});
            }
            return refs;
        }
        return IWordVecBuilder { inner, refs_builder }.build();
    }

    fn from_builder<F>(s : String, f : F) -> Self
    where F : FnOnce(&String)->Vec<&str>
    {
        return IWordVecBuilder{inner: s, refs_builder: f}.build();
    }
}

impl<'a> FromIterator<&'a str> for IWordVec
{
    fn from_iter<T>(iter: T) -> Self 
        where T : IntoIterator<Item=&'a str>
    {
        let mut inner = String::new();
        let mut intermediate = vec![];
        iter.into_iter().for_each(|s| {
            let prev = inner.len();
            inner.push_str(s);
            intermediate.push(prev..inner.len());
        }); 
        
        return IWordVecBuilder{inner, refs_builder: |inner : &String| {
            intermediate.iter()
                        .map(|range| &inner[range.start..range.end])
                        .collect::<Vec<&str>>()
        }}.build();
    }
}

impl Index<usize> for IWordVec 
{
    type Output = str;
    fn index(&self, index:usize)-> &str 
    {
        return &self.borrow_refs()[index];
    }
}

//could impl iterator https://github.com/pretzelhammer/rust-blog/blob/master/posts/tour-of-rusts-standard-library-traits.md#iteration-traits

#[derive(Debug)]
pub struct WordVec 
{
    inner : IWordVec
}

impl WordVec 
{
    pub fn from_csv(src : String) -> Self{
        return Self { inner: IWordVec::from_csv(src) };
    }

    pub fn from_builder<F>(s : String, f : F) -> Self
    where F : FnOnce(&String)->Vec<&str>
    {    
        return Self{inner: IWordVec::from_builder(s, f)};
    }

    fn len(&self)-> usize 
    {
        return self.inner.borrow_refs().len()
    }
}

impl<'a> FromIterator<&'a str> for WordVec
{
    fn from_iter<T>(iter: T) -> Self 
        where T : IntoIterator<Item=&'a str>
    {
        return Self{ inner: IWordVec::from_iter(iter) };
    }
}

impl Index<usize> for WordVec 
{
    type Output = str;
    fn index(&self, index:usize)-> &str 
    {
        return &self.inner.borrow_refs()[index];
    }
}


#[cfg(test)]
mod libtest
{
    use super::*;
    extern crate test;
    use test::Bencher;
    use crate::trallocator::TracedAllocator;
    use std::alloc::System;

    #[global_allocator]
    static GLOBAL: TracedAllocator<System> = TracedAllocator::new(System);


    #[test]
    fn full_test()
    {
        let sample = "cell0, \"cell1\", cell2,\n\"cell\n3\",\t\"cell4\"";
        let sample2 = "a b c d e\nf g h i ";
        let csv_words = WordVec::from_csv(sample.to_string());
        let builder_words = WordVec::from_builder(sample2.to_string(), |s| s.split_whitespace().collect()); 
        let iter_words = WordVec::from_iter(["a","b","c"]);
        let test_eq = |a : &[&str], b : &WordVec| {
            assert_eq!(a.len(), b.len());
            (0..a.len()).for_each(|i| assert!(a[i] == &b[i], "{} != {}",a[i], &b[i]));
        };
        test_eq(&["cell0", "cell1", "cell2", "cell\n3", "cell4"], &csv_words);
        test_eq(&["a", "b", "c", "d", "e", "f", "g", "h", "i"],&builder_words);
        test_eq(&["a", "b", "c"], &iter_words);
    }

    //ok speed difference is nothing until we fill up cpu cache which won't happen.
    #[bench]
    fn worth(b : &mut Bencher)
    {
        let txt = load_words().repeat(1000);
        let wordvec = WordVec::from_csv(txt);
        let samples = (0..10_000).map(|_|rand::random_range(0..wordvec.len())).collect::<Vec<usize>>();
        b.iter(|| {
            samples.iter().map(|idx| &wordvec[*idx]).count();
        });
    }

    //16kb
    #[test]
    fn test_mem()
    {
        let txt = load_words();
        let wv = |txt: String| WordVec::from_csv(txt);
        println!("{:?}", bench_mem(wv, txt).1);
    }

    //22.5kb 
    #[test]
    fn test_mem_ctrl()
    {
        let txt = load_words();
        let wv = |txt: String| txt.split(", ").map(|s| s.to_string()).collect::<Vec<String>>();
        println!("{:?}", bench_mem(wv, txt).1);
    }   

    fn bench_mem<T,I, F: FnOnce(I)->T>(f: F, arg: I) -> (T, u64, std::time::Duration)
    {
        GLOBAL.reset();
        let mem0 = GLOBAL.get();
        let start = std::time::Instant::now();
        let r = f(arg);
        let mem1 = GLOBAL.get();
        let end = std::time::Instant::now();
        return (r, mem1 - mem0, end - start);
    }
}