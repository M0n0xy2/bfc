extern crate bfc;
extern crate clap;

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use clap::{Arg, App};

use bfc::{ir, opt, interpreter, c_backend, llvm_backend};

fn main() {
    let matches = App::new("Brainfuck Compiler")
        .version("0.1")
        .author("Paul CACHEUX <paulcacheux@gmail.com>")
        .about("Interpret brainfuck programs")
        .arg(Arg::with_name("opt")
             .short("O")
             .help("Activate optimizations"))
        .arg(Arg::with_name("type")
             .help("Choose compilation type")
             .short("t")
             .long("type")
             .takes_value(true)
             .possible_values(&["c", "interpreter", "jit"])
             .requires_if("c", "OUTPUT"))
        .arg(Arg::with_name("INPUT")
             .help("Input file")
             .required(true)
             .index(1))
        .arg(Arg::with_name("ir")
             .help("Print ir to stdout"))
        .arg(Arg::with_name("OUTPUT")
            .help("Output file")
            .short("o")
            .takes_value(true))
        .get_matches();

    let path = matches.value_of("INPUT").unwrap();
    let buf = slurp_file(path).unwrap();
    let mut ir = ir::build_ir(&buf).unwrap();
    let opt = matches.is_present("opt");
    if opt {
        ir = opt::run_opts(ir);
    }
    
    if matches.is_present("ir") {
        println!("{:#?}", ir);
    }

    match matches.value_of("type") {
        Some("interpreter") | None => {
            if let Err(err) = interpreter::interpret(&ir, io::stdin(), io::stdout()) {
                println!("Interpreting finished with error: {:?}", err);
            }
        },
        Some("c") => {
            let output_path = matches.value_of("OUTPUT").unwrap();
            if let Err(err) = write_c(output_path, &ir) {
                println!("Error while writing C file: {}", err);
            }
        },
        Some("jit") => {
            if let Err(err) = llvm_backend::jit_ir(&ir, opt) {
                println!("LLVM Error: {:?}", err);
            }
        }
        _ => unreachable!()
    }
}

fn slurp_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

fn write_c<P: AsRef<Path>>(path: P, ir: &Vec<ir::Atom>) -> io::Result<()> {
    let output_file = File::create(path)?;
    c_backend::write_from_ir(ir, output_file)
}
