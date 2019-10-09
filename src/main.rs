/// Simulate a LRU CPU cache

use std::env;
use std::fs;
use std::error::Error;
use getopts::Options;

mod valgrind;
mod cache;

fn main() -> Result<(), Box<dyn Error>> {
    let argv: Vec<String> = env::args().skip(1).collect();

    let mut opts = Options::new();
    opts.reqopt("s", "set", "Number of set index bits", "");
    opts.reqopt("E", "lines", "Number of lines per set", "");
    opts.reqopt("b", "block", "Number of block bits", "");
    opts.reqopt("f", "file", "File containing instruction accesses", "");

    let matches = match opts.parse(&argv) {
        Ok(m) => m,
        Err(_) => {
            print_usage();
            return Ok(());
        }
    };

    let file_contents: String = fs::read_to_string(matches.opt_str("f").unwrap())?;
    let traces = valgrind::parse(&file_contents)?;

    let set_bits = matches.opt_str("s").unwrap().parse::<u8>()?;
    let lines = matches.opt_str("E").unwrap().parse::<u8>()?;
    let block_bits = matches.opt_str("b").unwrap().parse::<u8>()?;

    let mut cache = cache::Cache::new(set_bits, lines, block_bits);
    cache.operate_cache(traces);

    dbg!(cache.stats);
    Ok(())
}

fn print_usage() {
    let usage = "Usage: csim -s <num> -E <num> -b <num> -f <file>\n\
    Options:
        -s <num>  Number of set index bits.
        -E <num>  Number of lines per set.
        -b <num>  Number of lines per set.
        -f <file> Valgrind instruction log.\
    ";
    println!("{}", usage);
}
