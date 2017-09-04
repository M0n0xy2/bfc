extern crate itertools;
#[cfg(test)]
extern crate quickcheck;

pub mod utils;
pub mod ir;
pub mod opt;
pub mod interpreter;

#[cfg(test)]
mod tests {
    use super::{ir, interpreter, opt};
    use ir::Atom;
    use quickcheck::{quickcheck, TestResult};
    use std::io::Cursor;

    const LOOP_LIMIT: usize = 255 * 4;

    fn get_output(ir: &Vec<Atom>, input: &Vec<u8>) -> Result<Vec<u8>, String> {
        let mut output_buf = Cursor::new(Vec::<u8>::new());
        let result = interpreter::interpret_with_loop_limit(
            &ir,
            Cursor::new(input),
            &mut output_buf,
            LOOP_LIMIT
        );

        match result {
            Ok(_) => Ok(output_buf.into_inner()),
            Err(err) => Err(format!("{:?}", err)),
        }
    }

    #[test]
    fn quickcheck_opt_no_change() {
        fn opt_no_change(prog: Vec<u8>, input: Vec<u8>) -> TestResult {
            let ir = if let Ok(ir) = ir::build_ir(&prog) {
                ir
            } else {
                return TestResult::discard();
            };

            const MAX_PROG_SIZE: usize = 1_000_000;
            if prog.len() >= MAX_PROG_SIZE {
                return TestResult::discard();
            }

            let opt_ir = opt::run_opts(ir.clone());
            let normal_output = get_output(&ir, &input);
            let opt_output = get_output(&opt_ir, &input);

            TestResult::from_bool(normal_output == opt_output)
        }

        quickcheck(opt_no_change as fn(Vec<u8>, Vec<u8>) -> TestResult);
    }
}
