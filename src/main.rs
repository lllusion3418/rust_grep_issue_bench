#![feature(test)]

#[cfg(test)]
extern crate test;

#[cfg(test)]
use test::Bencher;

use std::fs;
use std::time::Instant;

use grep::printer;
use grep::regex;
use grep::searcher;

fn setup() -> (
    Vec<u8>,
    regex::RegexMatcher,
    searcher::Searcher,
    printer::JSON<fs::File>,
) {
    let mut slice = Vec::new();
    for _ in 0..100 {
        slice.extend(b"aaaaaaaaaaaaaaaa\n");
    }
    /* also happens with
     * - "{", "}" (ASCII)
     * - e.g. "§", "ü" (non-ASCII)
     */
    slice.extend("|".as_bytes());

    // also happens with a dummy sink implementation which does nothing
    let devnull = fs::OpenOptions::new().write(true).open("/dev/null").unwrap();
    let printer = printer::JSON::new(devnull);

    let matcher = regex::RegexMatcherBuilder::new()
        /* variations for which it also does happen:
         * - \bfoo\b
         * variations for which it doesn't happen:
         * - \bfoo
         * - foo\b
         * - foo(bar|baz)
         * - (bar|baz)foo(bar|baz)
         * - (?-u:\b)foo(bar|baz)
         */
        .build(r"\bfoo(bar|baz)")
        .unwrap();

    let searcher = searcher::SearcherBuilder::new()
        .encoding(None)
        .bom_sniffing(false)
        .build();

    (slice, matcher, searcher, printer)
}

#[cfg(test)]
#[bench]
fn search_slice(b: &mut Bencher) {
    let (slice, matcher, mut searcher, mut printer) = setup();
    b.iter(|| {
        searcher
            .search_slice(&matcher, &slice[..], printer.sink(&matcher))
            .unwrap();
    });
}

#[cfg(test)]
#[bench]
fn search_reader(b: &mut Bencher) {
    let (slice, matcher, mut searcher, mut printer) = setup();
    b.iter(|| {
        searcher
            .search_reader(&matcher, &slice[..], printer.sink(&matcher))
            .unwrap();
    });
}

fn main() {
    let (slice, matcher, mut searcher, mut printer) = setup();
    
    eprintln!("# search_reader");
    let start = Instant::now();
    searcher
        .search_reader(&matcher, &slice[..], printer.sink(&matcher))
        .unwrap();
    eprintln!("- {:?}", start.elapsed());
    
    eprintln!("# search_slice");
    let start = Instant::now();
    searcher
        .search_slice(&matcher, &slice[..], printer.sink(&matcher))
        .unwrap();
    eprintln!("- {:?}", start.elapsed());
}