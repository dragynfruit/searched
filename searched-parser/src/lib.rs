//extern crate combine;

use std::{collections::HashMap, ops::Range};

//use combine::{between, choice, many, satisfy, token, Parser};

#[derive(Clone, Debug)]
enum UrlElem {
    /// Range for a static string
    Static(Range<usize>),
    /// Range for a dynamic key
    Dynamic(Range<usize>),
}

#[derive(Clone)]
pub struct Url {
    src: Vec<u8>,
    elems: Vec<UrlElem>,
}
impl Url {
    pub fn parse(src: &[u8]) -> Self {
        let mut elems = Vec::new();

        let len = src.len();
        let mut pos = 0usize;
        let mut st = 0usize;

        let mut unclosed = false;

        while pos < len {
            if src[pos] != b'{' && src[pos] != b'}' {
                pos += 1;
            } else {
                if src[pos] == b'{' {
                    assert!(
                        !unclosed,
                        "can't open another dynamic before closing the last one"
                    );
                    elems.push(UrlElem::Static(st..pos));
                } else {
                    assert!(unclosed, "can't close before opening a dynamic");
                    elems.push(UrlElem::Dynamic(st..pos));
                }

                unclosed = !unclosed;

                st = pos + 1;
                pos = st;
            }
        }

        Self {
            src: src.to_vec(),
            elems,
        }
    }
    pub fn build(&self, values: HashMap<String, String>) -> String {
        self.elems
            .iter()
            .map(|elem| match elem {
                UrlElem::Static(r) => {
                    core::str::from_utf8(self.src.get(r.clone()).unwrap()).unwrap()
                }
                UrlElem::Dynamic(r) => values
                    .get(core::str::from_utf8(self.src.get(r.clone()).unwrap()).unwrap())
                    .unwrap(),
            })
            .collect::<Vec<&str>>()
            .concat()
            .to_string()
    }
}

//pub struct Query {
//}
//impl Query {
//    pub fn parse() {
//        many(choice([
//            between(token('"'), token('"'), many(satisfy(|c| c != '"'))),
//        ])).parse("").unwrap();
//    }
//}
