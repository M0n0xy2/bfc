extern crate bfc;
extern crate clap;

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use clap::{Arg, App};

use bfc::{ir, opt, interpreter};

fn slurp_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

fn main() {
    let matches = App::new("Brainfuck Compiler")
        .version("0.1")
        .author("Paul CACHEUX <paulcacheux@gmail.com>")
        .about("Interpret brainfuck programs")
        .arg(Arg::with_name("opt")
             .short("O")
             .help("Activate optimizations"))
        .arg(Arg::with_name("INPUT")
             .help("Input file")
             .required(true)
             .index(1))
        .get_matches();

    let path = matches.value_of("INPUT").unwrap();
    let buf = slurp_file(path).unwrap();
    let mut ir = ir::build_ir(&buf).unwrap();
    if matches.is_present("opt") {
        ir = opt::run_opts(ir);
    }

    println!("{:?}", ir);

    println!("--> INTERPRETING");
    println!("{:?}", interpreter::interpret(&ir, io::stdin(), io::stdout()));
}
