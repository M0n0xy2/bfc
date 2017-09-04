use itertools::Itertools;

use ir::Atom;
use ir::Atom::*;

pub fn run_opts(mut ir: Vec<Atom>) -> Vec<Atom> {
    let opts = [
        combine,
        clean,
        zero_loops,
        offset_op,
        reorder,
        add_multiply,
        combine,
        clean
    ];

    for opt in &opts {
        ir = opt(ir);
    }
    ir
}

fn combine(ir: Vec<Atom>) -> Vec<Atom> {
    fn combiner(a: Atom, b: Atom) -> Result<Atom, (Atom, Atom)> {
        match (a, b) {
            (MovePtr(av), MovePtr(bv)) => Ok(MovePtr(av.wrapping_add(bv))),
            (IncValue(av, o1), IncValue(bv, o2)) if o1 == o2 => {
                Ok(IncValue(av.wrapping_add(bv), o1))
            },
            (IncValue(_, o1), SetValue(sv, o2)) if o1 == o2 => {
                Ok(SetValue(sv, o1))
            },
            (SetValue(sv, o1), IncValue(ov, o2)) if o1 == o2 => {
                Ok(SetValue(sv.wrapping_add(ov), o1))
            },
            (SetValue(_, o1), SetValue(sv, o2)) if o1 == o2 => {
                Ok(SetValue(sv, o1))
            },
            (a, b) => Err((a, b))
        }
    }

    ir.into_iter().map(|atom| {
        if let Loop(sub) = atom {
            Loop(combine(sub))
        } else {
            atom
        }
    }).coalesce(combiner).collect()
}

fn zero_loops(ir: Vec<Atom>) -> Vec<Atom> {
    ir.into_iter().map(|atom| {
        if let Atom::Loop(sub) = atom {
            let new_sub = zero_loops(sub);
            if new_sub == [Atom::IncValue(-1, 0)] {
                Atom::SetValue(0, 0)
            } else {
                Atom::Loop(new_sub)
            }
        } else {
            atom
        }
    }).collect()
}

fn clean(ir: Vec<Atom>) -> Vec<Atom> {
    ir.into_iter().filter_map(|atom| {
        match atom {
            MovePtr(0) | IncValue(0, _) => None,
            Loop(content) => Some(Loop(clean(content))),
            other => Some(other),
        }
    }).collect()
}

fn offset_op(ir: Vec<Atom>) -> Vec<Atom> {
    let mut new_ir = Vec::with_capacity(ir.len());
    
    let mut current_offset = 0isize;
    for atom in ir {
        match atom {
            MovePtr(offset) => {
                current_offset = current_offset.wrapping_add(offset);
            },
            SetValue(value, offset) => {
                let new_offset = current_offset.wrapping_add(offset);
                new_ir.push(SetValue(value, new_offset));
            },
            IncValue(inc, offset) => {
                let new_offset = current_offset.wrapping_add(offset);
                new_ir.push(IncValue(inc, new_offset));
            },
            Print(offset) => {
                new_ir.push(Print(current_offset.wrapping_add(offset)));
            },
            Read(offset) => {
                new_ir.push(Read(current_offset.wrapping_add(offset)));
            },
            Multiply(factor, offset) => {
                new_ir.push(MovePtr(current_offset));
                current_offset = 0;

                new_ir.push(Multiply(factor, offset));
            },
            Loop(sub) => {
                new_ir.push(MovePtr(current_offset));
                current_offset = 0;

                new_ir.push(Loop(offset_op(sub)));
            },
        }
    }
    new_ir.push(MovePtr(current_offset));
    new_ir
}

fn reorder(ir: Vec<Atom>) -> Vec<Atom> {
    fn offset_extractor(atom: &Atom) -> isize {
        match *atom {
            SetValue(_, offset) => offset,
            IncValue(_, offset) => offset,
            _ => unreachable!(),
        }
    }

    let mut new_ir = Vec::with_capacity(ir.len());
    let mut temp_ir = Vec::new();

    for atom in ir {
        let atom = if let Atom::Loop(sub) = atom {
            Atom::Loop(reorder(sub))
        } else {
            atom
        };

        match atom {
            a@MovePtr(_) |
            a@Print(_) |
            a@Read(_) |
            a@Multiply(_, _) |
            a@Loop(_) => {
                temp_ir.sort_by_key(offset_extractor);
                new_ir.extend(temp_ir.drain(..));
                new_ir.push(a);
            },
            other => {
                temp_ir.push(other);
            }
        }
    }
    temp_ir.sort_by_key(offset_extractor);
    new_ir.extend(temp_ir.into_iter());
    new_ir
}

fn add_multiply(ir: Vec<Atom>) -> Vec<Atom> {
    // really returns a Vec<Atom> to be directly extended in upper "loop"
    fn work_on_loop(loop_content: Vec<Atom>) -> Vec<Atom> {
        use std::collections::HashMap;

        let save = loop_content.clone();

        let mut total_ptr_offset = 0isize;
        let mut increments = HashMap::new();
        for atom in loop_content {
            match atom {
                MovePtr(offset) => {
                    total_ptr_offset = total_ptr_offset.wrapping_add(offset);
                },
                IncValue(inc, offset) => {
                    let old = increments.entry(
                        offset.wrapping_add(total_ptr_offset)
                    ).or_insert(0i8);
                    *old = old.wrapping_add(inc);
                }
                _ => return vec![Atom::Loop(save)]
            }
        }

        if let Some(&-1) = increments.get(&0) {
            if total_ptr_offset == 0 {
                let mut atoms: Vec<_> = increments
                    .into_iter()
                    .filter_map(|(offset, inc)| {
                        if offset == 0 {
                            None
                        } else {
                            Some(Atom::Multiply(inc, offset))
                        }
                    }).collect();
                atoms.push(Atom::SetValue(0, 0));
                return atoms;
            }
        }

        return vec![Atom::Loop(save)]
    }

    let mut new_ir = Vec::with_capacity(ir.len());
    for atom in ir {
        if let Atom::Loop(sub) = atom {
            let sub = add_multiply(sub);
            new_ir.extend(work_on_loop(sub));
        } else {
            new_ir.push(atom);
        }
    }
    new_ir
}
