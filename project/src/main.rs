use std::{array, iter};

use bumpalo::Bump;
use hdl::{
    create_subchip, ArrayInto, ChipInput, ChipOutput, ChipOutputType, Input, Machine, Nand,
    NandInputs, SizedChip, UserInput,
};
use hdl_macro::{chip, StructuredData};

#[derive(StructuredData, PartialEq, Debug)]
struct UnaryChipOutput<T> {
    out: T,
}

#[derive(StructuredData, PartialEq, Debug)]
struct BinaryChipOutput<T> {
    out1: T,
    out2: T,
}

#[derive(StructuredData, PartialEq, Debug)]
struct ArrayLen2<T> {
    out: [T; 2],
}

#[derive(StructuredData, PartialEq, Debug)]
struct ArrayLen16<T> {
    out: [T; 16],
}

#[chip]
fn not<'a>(alloc: &'a Bump, in_: &'a ChipInput<'a>) -> UnaryChipOutput<ChipOutputType<'a>> {
    let nand = Nand::new(&alloc, in_.into(), in_.into());
    UnaryChipOutput { out: nand.into() }
}

#[chip]
fn and<'a>(
    alloc: &'a Bump,
    in1: &'a ChipInput<'a>,
    in2: &'a ChipInput<'a>,
) -> UnaryChipOutput<ChipOutputType<'a>> {
    let nand = Nand::new(&alloc, in1.into(), in2.into());
    let not = Not::new(alloc, nand.into());
    UnaryChipOutput {
        out: not.get_out(alloc).out.into(),
    }
}

#[chip]
fn or<'a>(
    alloc: &'a Bump,
    in1: &'a ChipInput<'a>,
    in2: &'a ChipInput<'a>,
) -> UnaryChipOutput<ChipOutputType<'a>> {
    let not1 = Not::new(&alloc, in1.into());
    let not2 = Not::new(&alloc, in2.into());
    let nand = Nand::new(
        &alloc,
        not1.get_out(alloc).out.into(),
        not2.get_out(alloc).out.into(),
    );
    UnaryChipOutput { out: nand.into() }
}

#[chip]
fn xor<'a>(
    alloc: &'a Bump,
    in1: &'a ChipInput<'a>,
    in2: &'a ChipInput<'a>,
) -> UnaryChipOutput<ChipOutputType<'a>> {
    let and = And::new(&alloc, in1.into(), in2.into());
    let not = Not::new(&alloc, and.get_out(alloc).out.into());
    let or = Or::new(&alloc, in1.into(), in2.into());
    let and2 = And::new(
        &alloc,
        not.get_out(alloc).out.into(),
        or.get_out(alloc).out.into(),
    );
    UnaryChipOutput {
        out: and2.get_out(alloc).out.into(),
    }
}

#[chip]
fn mux<'a>(
    alloc: &'a Bump,
    in1: &'a ChipInput<'a>,
    in2: &'a ChipInput<'a>,
    sel: &'a ChipInput<'a>,
) -> UnaryChipOutput<ChipOutputType<'a>> {
    let and1 = And::new(alloc, in2.into(), sel.into());
    let not = Not::new(alloc, sel.into());
    let and2 = And::new(alloc, in1.into(), not.get_out(alloc).out.into());
    let or = Or::new(
        alloc,
        and1.get_out(alloc).out.into(),
        and2.get_out(alloc).out.into(),
    );
    UnaryChipOutput {
        out: or.get_out(alloc).out.into(),
    }
}

#[chip]
fn demux<'a>(
    alloc: &'a Bump,
    in_: &'a ChipInput<'a>,
    sel: &'a ChipInput<'a>,
) -> BinaryChipOutput<ChipOutputType<'a>> {
    let and1 = And::new(alloc, in_.into(), sel.into());
    let not = Not::new(alloc, sel.into());
    let and2 = And::new(alloc, in_.into(), not.get_out(alloc).out.into());
    BinaryChipOutput {
        out1: and2.get_out(alloc).out.into(),
        out2: and1.get_out(alloc).out.into(),
    }
}

#[chip]
fn not16<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 16]) -> ArrayLen16<ChipOutputType<'a>> {
    // TODO: note that we can generalise this function to `NOT _n_`
    ArrayLen16 {
        out: input.map(|in_| Not::new(alloc, in_.into()).get_out(alloc).out.into()),
    }
}

fn zip<'a, T1, T2, const N: usize>(in1: [&'a T1; N], in2: [&'a T2; N]) -> [(&'a T1, &'a T2); N] {
    let mut out = [Option::None; N];
    for i in 0..N {
        out[i] = Some((in1[i], in2[i]));
    }
    out.map(|e| e.unwrap())
}

#[chip]
fn and16<'a>(
    alloc: &'a Bump,
    in1: [&'a ChipInput<'a>; 16],
    in2: [&'a ChipInput<'a>; 16],
) -> ArrayLen16<ChipOutputType<'a>> {
    let out = zip(in1, in2).map(|(in1, in2)| {
        And::new(alloc, in1.into(), in2.into())
            .get_out(alloc)
            .out
            .into()
    });
    ArrayLen16 { out }
}

#[chip]
fn or2<'a>(
    alloc: &'a Bump,
    in1: [&'a ChipInput<'a>; 2],
    in2: [&'a ChipInput<'a>; 2],
) -> ArrayLen2<ChipOutputType<'a>> {
    let out = zip(in1, in2).map(|(in1, in2)| {
        Or::new(alloc, in1.into(), in2.into())
            .get_out(alloc)
            .out
            .into()
    });
    ArrayLen2 { out }
}

#[chip]
fn mux16<'a>(
    alloc: &'a Bump,
    in1: [&'a ChipInput<'a>; 16],
    in2: [&'a ChipInput<'a>; 16],
    sel: &'a ChipInput<'a>,
) -> ArrayLen16<ChipOutputType<'a>> {
    let out = zip(in1, in2).map(|(in1, in2)| {
        Mux::new(
            alloc,
            Input::ChipInput(in1),
            Input::ChipInput(in2),
            Input::ChipInput(sel),
        )
        .get_out(alloc)
        .out
        .into()
    });
    ArrayLen16 { out }
}

#[chip]
fn andmult4<'a>(
    alloc: &'a Bump,
    in_: [&'a ChipInput<'a>; 4],
) -> UnaryChipOutput<ChipOutputType<'a>> {
    let initial_and = And::new(alloc, in_[0].into(), in_[1].into())
        .get_out(alloc)
        .out;
    let out = in_.iter().skip(2).fold(initial_and, |acc, in_| {
        And::new(alloc, (*in_).into(), acc.into())
            .get_out(alloc)
            .out
    });
    UnaryChipOutput { out: out.into() }
}

#[chip]
fn ormult16<'a>(
    alloc: &'a Bump,
    in_: [&'a ChipInput<'a>; 16],
) -> UnaryChipOutput<ChipOutputType<'a>> {
    let initial_nor = Or::new(alloc, in_[0].into(), in_[1].into());
    let out = in_.iter().skip(2).fold(initial_nor, |acc, in_| {
        Or::new(alloc, (*in_).into(), acc.get_out(alloc).out.into())
    });
    UnaryChipOutput {
        out: out.get_out(alloc).out.into(),
    }
}

#[derive(StructuredData, PartialEq, Debug)]
struct AdderOut<T> {
    sum: T,
    carry: T,
}

#[chip]
fn halfadder<'a>(
    alloc: &'a Bump,
    num1: &'a ChipInput<'a>,
    num2: &'a ChipInput<'a>,
) -> AdderOut<ChipOutputType<'a>> {
    let sum_bit = Xor::new(alloc, num1.into(), num2.into());
    let carry_bit = And::new(alloc, num1.into(), num2.into());
    AdderOut {
        carry: carry_bit.get_out(alloc).out.into(),
        sum: sum_bit.get_out(alloc).out.into(),
    }
}

#[chip]
fn fulladder<'a>(
    alloc: &'a Bump,
    num1: &'a ChipInput<'a>,
    num2: &'a ChipInput<'a>,
    num3: &'a ChipInput<'a>,
) -> AdderOut<ChipOutputType<'a>> {
    let first_hadder = Halfadder::new(alloc, num1.into(), num2.into());
    let second_hadder = Halfadder::new(alloc, num3.into(), first_hadder.get_out(alloc).sum.into());
    let carry_or = Or::new(
        alloc,
        first_hadder.get_out(alloc).carry.into(),
        second_hadder.get_out(alloc).carry.into(),
    );
    AdderOut {
        carry: carry_or.get_out(alloc).out.into(),
        sum: second_hadder.get_out(alloc).sum.into(),
    }
}

#[chip]
fn adder16<'a>(
    alloc: &'a Bump,
    num1: [&'a ChipInput<'a>; 16],
    num2: [&'a ChipInput<'a>; 16],
) -> ArrayLen16<ChipOutputType<'a>> {
    let lsb = Halfadder::new(alloc, num1[15].into(), num2[15].into());
    let zipin = num1[..15]
        .iter()
        .zip(&num2[..15])
        .rev()
        .fold(vec![lsb.get_out(alloc)], |mut acc, x| {
            let prev_carry = acc.last().unwrap().carry;
            let adder = Fulladder::new(alloc, prev_carry.into(), (*x.0).into(), (*x.1).into());
            acc.push(adder.get_out(alloc));
            acc
        })
        .iter()
        .map(|out| out.sum.into())
        .rev()
        .collect::<Vec<_>>();

    ArrayLen16 {
        out: zipin
            .try_into()
            .unwrap_or_else(|_| panic!("output must be exactly half of input")),
    }
}

#[chip]
fn incrementer16<'a>(
    alloc: &'a Bump,
    num: [&'a ChipInput<'a>; 16],
) -> ArrayLen16<ChipOutputType<'a>> {
    let inputs = num.map(|in_| Input::ChipInput(in_));
    let adder_inputs = iter::repeat_with(|| UserInput::from(alloc, false).into())
        .take(15)
        .chain(iter::once(UserInput::from(alloc, true).into()))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap_or_else(|_| panic!("array must be length 16"));
    let adder = Adder16::new(alloc, adder_inputs, inputs);
    let out = adder.get_out(alloc).out.ainto();
    ArrayLen16 { out }
}

#[derive(StructuredData, PartialEq, Debug)]
struct AluOutputs<T> {
    out: [T; 16],
    zr: T,
    ng: T,
}

#[chip]
fn zeronum<'a>(
    alloc: &'a Bump,
    num: [&'a ChipInput<'a>; 16],
    zero: &'a ChipInput<'a>,
) -> ArrayLen16<ChipOutputType<'a>> {
    let not_zero = Not16::new(alloc, array::from_fn(|_| Input::ChipInput(zero)));
    let zero_num = And16::new(alloc, num.ainto(), not_zero.get_out(alloc).out.ainto());

    ArrayLen16 {
        out: zero_num.get_out(alloc).out.ainto(),
    }
}

#[chip]
fn negatenum<'a>(
    alloc: &'a Bump,
    num: [&'a ChipInput<'a>; 16],
    negate: &'a ChipInput<'a>,
) -> ArrayLen16<ChipOutputType<'a>> {
    let not = Not16::new(alloc, num.ainto());
    let mux_not_x = Mux16::new(
        alloc,
        num.ainto(),
        not.get_out(alloc).out.ainto(),
        negate.into(),
    ); // note: it might be more power efficient in real hardware to demux first rather than
       // mux at the end. I'm not a real engineer though, so I don't know
    ArrayLen16 {
        out: mux_not_x.get_out(alloc).out.ainto(),
    }
}

#[chip]
fn andorplus<'a>(
    alloc: &'a Bump,
    num1: [&'a ChipInput<'a>; 16],
    num2: [&'a ChipInput<'a>; 16],
    isadd: &'a ChipInput<'a>,
) -> ArrayLen16<ChipOutputType<'a>> {
    let add_nums = Adder16::new(alloc, num1.ainto(), num2.ainto());
    let and_nums = And16::new(alloc, num1.ainto(), num2.ainto());
    let mux = Mux16::new(
        alloc,
        and_nums.get_out(alloc).out.ainto(),
        add_nums.get_out(alloc).out.ainto(),
        isadd.into(),
    );
    ArrayLen16 {
        out: mux.get_out(alloc).out.ainto(),
    }
}

#[chip]
fn alu<'a>(
    alloc: &'a Bump,
    x: [&'a ChipInput<'a>; 16],
    y: [&'a ChipInput<'a>; 16],
    zx: &'a ChipInput<'a>,
    zy: &'a ChipInput<'a>,
    nx: &'a ChipInput<'a>,
    ny: &'a ChipInput<'a>,
    f: &'a ChipInput<'a>,
    no: &'a ChipInput<'a>,
) -> AluOutputs<ChipOutputType<'a>> {
    let zero_x = Zeronum::new(alloc, x.ainto(), zx.into());
    let zero_y = Zeronum::new(alloc, y.ainto(), zy.into());
    let not_x = Negatenum::new(alloc, zero_x.get_out(alloc).out.ainto(), nx.into());
    let not_y = Negatenum::new(alloc, zero_y.get_out(alloc).out.ainto(), ny.into());
    let func = Andorplus::new(
        alloc,
        not_x.get_out(alloc).out.ainto(),
        not_y.get_out(alloc).out.ainto(),
        f.into(),
    );
    let negate_result = Negatenum::new(alloc, func.get_out(alloc).out.ainto(), no.into());
    let is_non_zero = Ormult16::new(alloc, negate_result.get_out(alloc).out.ainto());
    let is_zero = Not::new(alloc, is_non_zero.get_out(alloc).out.into());
    AluOutputs {
        out: negate_result.get_out(alloc).out.ainto(),
        zr: is_zero.get_out(alloc).out.into(),
        ng: negate_result.get_out(alloc).out[0].into(),
    }
}

#[derive(StructuredData, PartialEq, Debug)]
struct LatchOutput<T> {
    q: T,
    nq: T,
}

#[chip]
fn srlatch<'a>(
    alloc: &'a Bump,
    s: &'a ChipInput<'a>,
    r: &'a ChipInput<'a>,
) -> LatchOutput<ChipOutputType<'a>> {
    let (cross_nand_1, cross_nand_2): (&Nand, &Nand) = create_subchip(
        alloc,
        &|(nandchip,)| NandInputs {
            in1: s.into(),
            in2: nandchip.into(),
        },
        &|(nandchip,)| NandInputs {
            in1: r.into(),
            in2: nandchip.into(),
        },
    );

    LatchOutput {
        q: cross_nand_1.into(),
        nq: cross_nand_2.into(),
    }
}

#[chip]
fn dlatch<'a>(
    alloc: &'a Bump,
    data: &'a ChipInput<'a>,
    enable: &'a ChipInput<'a>,
) -> LatchOutput<ChipOutputType<'a>> {
    let notd = Not::new(alloc, data.into());
    let nand1 = Nand::new(alloc, data.into(), enable.into());
    let nand2 = Nand::new(alloc, notd.get_out(alloc).out.into(), enable.into());
    let srlatch = Srlatch::new(alloc, nand1.into(), nand2.into());

    let srout = srlatch.get_out(alloc);
    LatchOutput {
        q: srout.q.into(),
        nq: srout.nq.into(),
    }
}

#[chip]
fn dflipflop<'a>(
    alloc: &'a Bump,
    data: &'a ChipInput<'a>,
    clock: &'a ChipInput<'a>,
) -> LatchOutput<ChipOutputType<'a>> {
    let invclock = Not::new(alloc, clock.into());
    let latch1 = Dlatch::new(alloc, data.into(), clock.into());
    let latch2 = Dlatch::new(
        alloc,
        latch1.get_out(alloc).q.into(),
        invclock.get_out(alloc).out.into(),
    );

    let latch2out = latch2.get_out(alloc);
    LatchOutput {
        q: latch2out.q.into(),
        nq: latch2out.nq.into(),
    }
}

#[chip]
fn bit<'a>(
    alloc: &'a Bump,
    in_: &'a ChipInput<'a>,
    load: &'a ChipInput<'a>,
    clock: &'a ChipInput<'a>,
) -> UnaryChipOutput<ChipOutputType<'a>> {
    let (dff, _): (&Dflipflop, &Mux) = create_subchip(
        alloc,
        &|(mux,)| DflipflopInputs {
            data: mux.get_out(alloc).out.into(),
            clock: clock.into(),
        },
        &|(dff,)| MuxInputs {
            in1: dff.get_out(alloc).q.into(),
            in2: in_.into(),
            sel: load.into(),
        },
    );
    UnaryChipOutput {
        out: dff.get_out(alloc).q.into(),
    }
}

#[chip]
fn register16<'a>(
    alloc: &'a Bump,
    in_: [&'a ChipInput<'a>; 16],
    load: &'a ChipInput<'a>,
    clock: &'a ChipInput<'a>,
) -> ArrayLen16<ChipOutputType<'a>> {
    let out = in_.map(|elem| {
        Bit::new(alloc, elem.into(), load.into(), clock.into())
            .get_out(alloc)
            .out
            .into()
    });
    ArrayLen16 { out }
}

#[cfg(test)]
mod tests {
    use bumpalo::Bump;
    use hdl::Machine;

    use crate::*;

    #[test]
    fn register16_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Register16::from);
        let res = machine.process(Register16Inputs {
            in_: [true; 16],
            load: true,
            clock: true,
        }); // initial state
        assert_eq!(res.out, [false; 16]);
        let res = machine.process(Register16Inputs {
            in_: [false; 16],
            load: true,
            clock: false,
        }); // tock
        assert_eq!(res.out, [true; 16]);
    }

    #[test]
    fn bit_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Bit::from);
        let res = machine.process(BitInputs {
            in_: true,
            load: true,
            clock: true,
        }); // initial state
        assert_eq!(res.out, false);
        let res = machine.process(BitInputs {
            in_: true,
            load: true,
            clock: false,
        }); // tock
        assert_eq!(res.out, true);
        let res = machine.process(BitInputs {
            in_: true,
            load: true,
            clock: false,
        }); // same tock
        assert_eq!(res.out, true);
        let res = machine.process(BitInputs {
            in_: false,
            load: false,
            clock: true,
        }); // tick
        assert_eq!(res.out, true);
        let res = machine.process(BitInputs {
            in_: false,
            load: false,
            clock: false,
        }); // tock
        assert_eq!(res.out, true);
        let res = machine.process(BitInputs {
            in_: false,
            load: true,
            clock: true,
        }); // tick
        assert_eq!(res.out, true);
        let res = machine.process(BitInputs {
            in_: false,
            load: true,
            clock: false,
        }); // tock
        assert_eq!(res.out, false);
    }

    #[test]
    fn dflipflop_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Dflipflop::from);
        let res = machine.process(DflipflopInputs {
            data: true,
            clock: true,
        });
        assert_eq!(res.q, false, "q should not transition until a clock tick ");
        let res = machine.process(DflipflopInputs {
            data: false,
            clock: false,
        });
        assert_eq!(res.q, true, "data should transition on a clock tick");
        let res = machine.process(DflipflopInputs {
            data: false,
            clock: false,
        });
        assert_eq!(res.q, true, "data should not transition until a clock tick");
        let res = machine.process(DflipflopInputs {
            data: false,
            clock: true,
        });
        assert_eq!(
            res.q, true,
            "data should not transition until a clock tick after it was changed"
        );
        let res = machine.process(DflipflopInputs {
            data: false,
            clock: false,
        });
        assert_eq!(res.q, false, "data should transition on a clock tick");
        let res = machine.process(DflipflopInputs {
            data: false,
            clock: false,
        });
        assert_eq!(
            res.q, false,
            "data should not transition until a clock tick"
        );
    }

    #[test]
    fn dlatch_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Dlatch::from);
        let res = machine.process(DlatchInputs {
            data: true,
            enable: true,
        });
        assert_eq!(res.q, true);
        let res = machine.process(DlatchInputs {
            data: false,
            enable: false,
        });
        assert_eq!(res.q, true);
        let res = machine.process(DlatchInputs {
            data: false,
            enable: true,
        });
        assert_eq!(res.q, false);
    }

    #[test]
    fn srlatch_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Srlatch::from);
        let res1 = machine.process(SrlatchInputs { s: false, r: true });
        assert_eq!(res1.q, true);
        let res2 = machine.process(SrlatchInputs { s: true, r: true });
        assert_eq!(res2.q, true);
        let res3 = machine.process(SrlatchInputs { s: true, r: false });
        assert_eq!(res3.q, false);
        let res4 = machine.process(SrlatchInputs { s: true, r: true });
        assert_eq!(res4.q, false);
    }

    #[test]
    fn srlatch_has_stable_output_if_input_is_valid() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Srlatch::from);
        let res1 = machine.process(SrlatchInputs { s: false, r: true });
        assert_eq!(res1.q, true);
        let res2 = machine.process(SrlatchInputs { s: true, r: true });
        assert_eq!(res2.q, true);
        let res4 = machine.process(SrlatchInputs { s: true, r: true });
        assert_eq!(res4.q, true);
    }

    #[test]
    fn alu_chip_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Alu::from);

        // addition works
        let res = machine.process(AluInputs {
            x: [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, false, true,
            ],
            y: [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, false, true,
            ],
            zx: false,
            zy: false,
            ny: false,
            nx: false,
            f: true,
            no: false,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [
                    false, false, false, false, false, false, false, false, false, false, false,
                    false, false, false, true, false
                ],
                zr: false,
                ng: false
            }
        );

        // zx works
        let res = machine.process(AluInputs {
            x: [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, false, true,
            ],
            y: [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, true, false,
            ],
            zx: true,
            zy: false,
            ny: false,
            nx: false,
            f: true,
            no: false,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [
                    false, false, false, false, false, false, false, false, false, false, false,
                    false, false, false, true, false
                ],
                zr: false,
                ng: false
            }
        );

        // zy works
        let res = machine.process(AluInputs {
            x: [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, false, true,
            ],
            y: [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, true, false,
            ],
            zx: false,
            zy: true,
            ny: false,
            nx: false,
            f: true,
            no: false,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [
                    false, false, false, false, false, false, false, false, false, false, false,
                    false, false, false, false, true
                ],
                zr: false,
                ng: false
            }
        );

        // nx works
        let res = machine.process(AluInputs {
            x: [false; 16],
            y: [true; 16],
            zx: false,
            zy: false,
            ny: false,
            nx: true,
            f: true,
            no: false,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [
                    true, true, true, true, true, true, true, true, true, true, true, true, true,
                    true, true, false
                ],
                zr: false,
                ng: true
            }
        );

        // ny works
        let res = machine.process(AluInputs {
            x: [true; 16],
            y: [false; 16],
            zx: false,
            zy: false,
            ny: true,
            nx: false,
            f: true,
            no: false,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [
                    true, true, true, true, true, true, true, true, true, true, true, true, true,
                    true, true, false
                ],
                zr: false,
                ng: true
            }
        );

        // no works
        let res = machine.process(AluInputs {
            x: [true; 16],
            y: [true; 16],
            zx: false,
            zy: false,
            ny: false,
            nx: false,
            f: true,
            no: true,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [
                    false, false, false, false, false, false, false, false, false, false, false,
                    false, false, false, false, true
                ],
                ng: false,
                zr: false
            }
        );

        // and works
        let res = machine.process(AluInputs {
            x: [true; 16],
            y: [true; 16],
            zx: false,
            zy: false,
            ny: false,
            nx: false,
            f: false,
            no: false,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [true; 16],
                ng: true,
                zr: false
            }
        );

        // now I'll just put in the rest of the truth table as per the book
        // 0
        let res = machine.process(AluInputs {
            x: [true; 16],
            y: [true; 16],
            zx: true,
            zy: true,
            ny: false,
            nx: false,
            f: true,
            no: false,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [false; 16],
                ng: false,
                zr: true
            }
        );

        // 1
        let res = machine.process(AluInputs {
            x: [true; 16],
            y: [true; 16],
            zx: true,
            zy: true,
            ny: true,
            nx: true,
            f: true,
            no: true,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [
                    false, false, false, false, false, false, false, false, false, false, false,
                    false, false, false, false, true
                ],
                ng: false,
                zr: false
            }
        );

        // -1
        let res = machine.process(AluInputs {
            x: [true; 16],
            y: [true; 16],
            zx: true,
            zy: true,
            ny: false,
            nx: true,
            f: true,
            no: false,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [true; 16],
                ng: true,
                zr: false
            }
        );

        // x
        let res = machine.process(AluInputs {
            x: [true; 16],
            y: [false; 16],
            zx: false,
            zy: true,
            ny: true,
            nx: false,
            f: false,
            no: false,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [true; 16],
                ng: true,
                zr: false
            }
        );

        // y
        let res = machine.process(AluInputs {
            x: [false; 16],
            y: [true; 16],
            zx: true,
            zy: false,
            ny: false,
            nx: true,
            f: false,
            no: false,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [true; 16],
                ng: true,
                zr: false
            }
        );

        // !x
        let res = machine.process(AluInputs {
            x: [true; 16],
            y: [false; 16],
            zx: false,
            zy: true,
            ny: true,
            nx: false,
            f: false,
            no: true,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [false; 16],
                ng: false,
                zr: true
            }
        );

        // !y
        let res = machine.process(AluInputs {
            x: [true; 16],
            y: [false; 16],
            zx: true,
            zy: false,
            ny: false,
            nx: true,
            f: false,
            no: true,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [true; 16],
                ng: true,
                zr: false
            }
        );

        // x+1
        let res = machine.process(AluInputs {
            x: [true; 16],
            y: [false; 16],
            zx: false,
            zy: true,
            ny: true,
            nx: true,
            f: true,
            no: true,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [false; 16],
                ng: false,
                zr: true
            }
        );

        // y+1
        let res = machine.process(AluInputs {
            x: [true; 16],
            y: [false; 16],
            zx: true,
            zy: false,
            ny: true,
            nx: true,
            f: true,
            no: true,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [
                    false, false, false, false, false, false, false, false, false, false, false,
                    false, false, false, false, true
                ],
                ng: false,
                zr: false
            }
        );

        // x-1
        let res = machine.process(AluInputs {
            x: [true; 16],
            y: [false; 16],
            zx: false,
            zy: true,
            ny: true,
            nx: false,
            f: true,
            no: false,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [
                    true, true, true, true, true, true, true, true, true, true, true, true, true,
                    true, true, false
                ],
                ng: true,
                zr: false
            }
        );

        // y-1
        let res = machine.process(AluInputs {
            x: [true; 16],
            y: [false; 16],
            zx: true,
            zy: false,
            ny: false,
            nx: true,
            f: true,
            no: false,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [true; 16],
                ng: true,
                zr: false
            }
        );

        // x+y
        let res = machine.process(AluInputs {
            x: [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, false, true,
            ],
            y: [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, true, false,
            ],
            zx: false,
            zy: false,
            ny: false,
            nx: false,
            f: true,
            no: false,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [
                    false, false, false, false, false, false, false, false, false, false, false,
                    false, false, false, true, true
                ],
                ng: false,
                zr: false
            }
        );

        // x-y
        let res = machine.process(AluInputs {
            x: [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, false, true,
            ],
            y: [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, true, false,
            ],
            zx: false,
            zy: false,
            ny: false,
            nx: true,
            f: true,
            no: true,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [true; 16],
                ng: true,
                zr: false
            }
        );

        // y-x
        let res = machine.process(AluInputs {
            x: [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, false, true,
            ],
            y: [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, true, false,
            ],
            zx: false,
            zy: false,
            ny: true,
            nx: false,
            f: true,
            no: true,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [
                    false, false, false, false, false, false, false, false, false, false, false,
                    false, false, false, false, true
                ],
                ng: false,
                zr: false
            }
        );

        // x|y
        let res = machine.process(AluInputs {
            x: [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, true, false, true,
            ],
            y: [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, true, false,
            ],
            zx: false,
            zy: false,
            ny: true,
            nx: true,
            f: false,
            no: true,
        });
        assert_eq!(
            res,
            AluOutputs {
                out: [
                    false, false, false, false, false, false, false, false, false, false, false,
                    false, false, true, true, true
                ],
                ng: false,
                zr: false
            }
        );
    }

    #[test]
    fn not_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Not::from);
        assert_eq!(
            machine.process(NotInputs { in_: true }),
            UnaryChipOutput { out: false }
        );
        assert_eq!(
            machine.process(NotInputs { in_: false }),
            UnaryChipOutput { out: true }
        );
    }

    #[test]
    fn and_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, And::from);
        assert_eq!(
            machine.process(AndInputs {
                in1: true,
                in2: true
            }),
            UnaryChipOutput { out: true }
        );
        assert_eq!(
            machine.process(AndInputs {
                in1: true,
                in2: false
            }),
            UnaryChipOutput { out: false }
        );
        assert_eq!(
            machine.process(AndInputs {
                in1: false,
                in2: true
            }),
            UnaryChipOutput { out: false }
        );
        assert_eq!(
            machine.process(AndInputs {
                in1: false,
                in2: false
            }),
            UnaryChipOutput { out: false }
        );
    }

    #[test]
    fn or_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Or::from);
        assert_eq!(
            machine.process(OrInputs {
                in1: true,
                in2: true
            }),
            UnaryChipOutput { out: true }
        );
        assert_eq!(
            machine.process(OrInputs {
                in1: true,
                in2: false
            }),
            UnaryChipOutput { out: true }
        );
        assert_eq!(
            machine.process(OrInputs {
                in1: false,
                in2: true
            }),
            UnaryChipOutput { out: true }
        );
        assert_eq!(
            machine.process(OrInputs {
                in1: false,
                in2: false
            }),
            UnaryChipOutput { out: false }
        );
    }

    #[test]
    fn xor_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Xor::from);
        assert_eq!(
            machine.process(XorInputs {
                in1: true,
                in2: true
            }),
            UnaryChipOutput { out: false }
        );
        assert_eq!(
            machine.process(XorInputs {
                in1: true,
                in2: false
            }),
            UnaryChipOutput { out: true }
        );
        assert_eq!(
            machine.process(XorInputs {
                in1: false,
                in2: true
            }),
            UnaryChipOutput { out: true }
        );
        assert_eq!(
            machine.process(XorInputs {
                in1: false,
                in2: false
            }),
            UnaryChipOutput { out: false }
        );
    }

    #[test]
    fn mux_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Mux::from);
        assert_eq!(
            machine.process(MuxInputs {
                in1: true,
                in2: true,
                sel: true
            }),
            UnaryChipOutput { out: true }
        );
        assert_eq!(
            machine.process(MuxInputs {
                in1: true,
                in2: false,
                sel: true
            }),
            UnaryChipOutput { out: false }
        );
        assert_eq!(
            machine.process(MuxInputs {
                in1: true,
                in2: true,
                sel: false
            }),
            UnaryChipOutput { out: true }
        );
        assert_eq!(
            machine.process(MuxInputs {
                in1: false,
                in2: true,
                sel: true
            }),
            UnaryChipOutput { out: true }
        );
        assert_eq!(
            machine.process(MuxInputs {
                in1: true,
                in2: false,
                sel: false
            }),
            UnaryChipOutput { out: true }
        );
        assert_eq!(
            machine.process(MuxInputs {
                in1: false,
                in2: true,
                sel: false
            }),
            UnaryChipOutput { out: false }
        );
        assert_eq!(
            machine.process(MuxInputs {
                in1: false,
                in2: false,
                sel: false
            }),
            UnaryChipOutput { out: false }
        );
        assert_eq!(
            machine.process(MuxInputs {
                in1: false,
                in2: false,
                sel: true
            }),
            UnaryChipOutput { out: false }
        );
    }

    #[test]
    fn demux_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Demux::from);
        assert_eq!(
            machine.process(DemuxInputs {
                in_: true,
                sel: true
            }),
            BinaryChipOutput {
                out1: false,
                out2: true
            }
        );
        assert_eq!(
            machine.process(DemuxInputs {
                in_: true,
                sel: false
            }),
            BinaryChipOutput {
                out1: true,
                out2: false
            }
        );
        assert_eq!(
            machine.process(DemuxInputs {
                in_: false,
                sel: true
            }),
            BinaryChipOutput {
                out1: false,
                out2: false
            }
        );
        assert_eq!(
            machine.process(DemuxInputs {
                in_: false,
                sel: false
            }),
            BinaryChipOutput {
                out1: false,
                out2: false
            }
        );
    }

    #[test]
    fn not16_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Not16::from);
        assert_eq!(
            machine.process(Not16Inputs { input: [true; 16] }),
            ArrayLen16 { out: [false; 16] }
        );
        assert_eq!(
            machine.process(Not16Inputs { input: [false; 16] }),
            ArrayLen16 { out: [true; 16] }
        );
    }

    #[test]
    fn and2_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, And16::from);
        assert_eq!(
            machine.process(And16Inputs {
                in1: [true; 16],
                in2: [true; 16]
            }),
            ArrayLen16 { out: [true; 16] }
        );
        assert_eq!(
            machine.process(And16Inputs {
                in1: [
                    false, true, true, true, true, true, true, true, true, true, true, true, true,
                    true, true, true
                ],
                in2: [
                    true, true, true, true, true, true, true, true, true, true, true, true, true,
                    true, true, true
                ]
            }),
            ArrayLen16 {
                out: [
                    false, true, true, true, true, true, true, true, true, true, true, true, true,
                    true, true, true
                ]
            }
        );
        assert_eq!(
            machine.process(And16Inputs {
                in1: [
                    true, false, true, true, true, true, true, true, true, true, true, true, true,
                    true, true, true
                ],
                in2: [
                    true, true, true, true, true, true, true, true, true, true, true, true, true,
                    true, true, true
                ]
            }),
            ArrayLen16 {
                out: [
                    true, false, true, true, true, true, true, true, true, true, true, true, true,
                    true, true, true
                ]
            }
        );
        assert_eq!(
            machine.process(And16Inputs {
                in1: [
                    true, true, true, true, true, true, true, true, true, true, true, true, true,
                    true, true, true
                ],
                in2: [
                    false, true, true, true, true, true, true, true, true, true, true, true, true,
                    true, true, true
                ]
            }),
            ArrayLen16 {
                out: [
                    false, true, true, true, true, true, true, true, true, true, true, true, true,
                    true, true, true
                ]
            }
        );
        assert_eq!(
            machine.process(And16Inputs {
                in1: [
                    true, true, true, true, true, true, true, true, true, true, true, true, true,
                    true, true, true
                ],
                in2: [
                    true, false, true, true, true, true, true, true, true, true, true, true, true,
                    true, true, true
                ]
            }),
            ArrayLen16 {
                out: [
                    true, false, true, true, true, true, true, true, true, true, true, true, true,
                    true, true, true
                ]
            }
        );
        // ...
        assert_eq!(
            machine.process(And16Inputs {
                in1: [false; 16],
                in2: [false; 16]
            }),
            ArrayLen16 { out: [false; 16] }
        );
    }

    #[test]
    fn or2_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Or2::from);
        assert_eq!(
            machine.process(Or2Inputs {
                in1: [true, true],
                in2: [true, true]
            }),
            ArrayLen2 { out: [true, true] }
        );
        assert_eq!(
            machine.process(Or2Inputs {
                in1: [false, true],
                in2: [true, true]
            }),
            ArrayLen2 { out: [true, true] }
        );
        assert_eq!(
            machine.process(Or2Inputs {
                in1: [true, false],
                in2: [true, true]
            }),
            ArrayLen2 { out: [true, true] }
        );
        assert_eq!(
            machine.process(Or2Inputs {
                in1: [true, true],
                in2: [false, true]
            }),
            ArrayLen2 { out: [true, true] }
        );
        assert_eq!(
            machine.process(Or2Inputs {
                in1: [true, true],
                in2: [true, false]
            }),
            ArrayLen2 { out: [true, true] }
        );
        // ...
        assert_eq!(
            machine.process(Or2Inputs {
                in1: [false, true],
                in2: [false, false]
            }),
            ArrayLen2 { out: [false, true] }
        );
        assert_eq!(
            machine.process(Or2Inputs {
                in1: [false, false],
                in2: [false, false]
            }),
            ArrayLen2 {
                out: [false, false]
            }
        );
    }

    #[test]
    fn mux16_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Mux16::from);
        assert_eq!(
            machine.process(Mux16Inputs {
                in1: [true; 16],
                in2: [false; 16],
                sel: true
            }),
            ArrayLen16 { out: [false; 16] }
        );
        assert_eq!(
            machine.process(Mux16Inputs {
                in1: [true; 16],
                in2: [false; 16],
                sel: false
            }),
            ArrayLen16 { out: [true; 16] }
        );
        // ...
    }

    #[test]
    fn andmult4_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Andmult4::from);
        assert_eq!(
            machine.process(Andmult4Inputs {
                in_: [true, true, true, true]
            }),
            UnaryChipOutput { out: true }
        );
        assert_eq!(
            machine.process(Andmult4Inputs {
                in_: [false, true, true, true]
            }),
            UnaryChipOutput { out: false }
        );
        assert_eq!(
            machine.process(Andmult4Inputs {
                in_: [true, false, true, true]
            }),
            UnaryChipOutput { out: false }
        );
        assert_eq!(
            machine.process(Andmult4Inputs {
                in_: [true, true, false, true]
            }),
            UnaryChipOutput { out: false }
        );
        assert_eq!(
            machine.process(Andmult4Inputs {
                in_: [true, true, true, false]
            }),
            UnaryChipOutput { out: false }
        );
        // ...
    }

    #[test]
    fn halfadder_chip_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Halfadder::from);
        assert_eq!(
            machine.process(HalfadderInputs {
                num1: false,
                num2: false
            }),
            AdderOut {
                sum: false,
                carry: false
            }
        );
        assert_eq!(
            machine.process(HalfadderInputs {
                num1: false,
                num2: true
            }),
            AdderOut {
                sum: true,
                carry: false
            }
        );
        assert_eq!(
            machine.process(HalfadderInputs {
                num1: true,
                num2: false
            }),
            AdderOut {
                sum: true,
                carry: false
            }
        );
        assert_eq!(
            machine.process(HalfadderInputs {
                num1: true,
                num2: true
            }),
            AdderOut {
                sum: false,
                carry: true
            }
        );
    }

    #[test]
    fn fulladder_chip_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Fulladder::from);

        assert_eq!(
            machine.process(FulladderInputs {
                num1: false,
                num2: false,
                num3: false
            }),
            AdderOut {
                sum: false,
                carry: false
            }
        );
        assert_eq!(
            machine.process(FulladderInputs {
                num1: false,
                num2: false,
                num3: true
            }),
            AdderOut {
                sum: true,
                carry: false
            }
        );
        assert_eq!(
            machine.process(FulladderInputs {
                num1: false,
                num2: true,
                num3: false
            }),
            AdderOut {
                sum: true,
                carry: false
            }
        );
        assert_eq!(
            machine.process(FulladderInputs {
                num1: true,
                num2: false,
                num3: false
            }),
            AdderOut {
                sum: true,
                carry: false
            }
        );
        assert_eq!(
            machine.process(FulladderInputs {
                num1: false,
                num2: true,
                num3: true
            }),
            AdderOut {
                sum: false,
                carry: true
            }
        );
        assert_eq!(
            machine.process(FulladderInputs {
                num1: true,
                num2: false,
                num3: true
            }),
            AdderOut {
                sum: false,
                carry: true
            }
        );
        assert_eq!(
            machine.process(FulladderInputs {
                num1: true,
                num2: true,
                num3: false
            }),
            AdderOut {
                sum: false,
                carry: true
            }
        );
        assert_eq!(
            machine.process(FulladderInputs {
                num1: true,
                num2: true,
                num3: true
            }),
            AdderOut {
                sum: true,
                carry: true
            }
        );
    }

    #[test]
    fn adder16_chip_has_correct_partial_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Adder16::from);

        assert_eq!(
            machine.process(Adder16Inputs {
                num1: [false; 16],
                num2: [false; 16]
            }),
            ArrayLen16 { out: [false; 16] },
            "0+0 != 0"
        );

        // check LSB and MSB values are represented
        let mut num1 = [false; 16];
        num1[15] = true;
        let mut num2 = [false; 16];
        num2[0] = true;
        let mut out = [false; 16];
        out[0] = true;
        out[15] = true;
        assert_eq!(
            machine.process(Adder16Inputs { num1, num2 }),
            ArrayLen16 { out },
            "1+32768 != 32769"
        );

        // check halfadder carry
        let mut num1 = [false; 16];
        num1[15] = true;
        let mut num2 = [false; 16];
        num2[15] = true;
        let mut out = [false; 16];
        out[15 - 1] = true;
        assert_eq!(
            machine.process(Adder16Inputs { num1, num2 }),
            ArrayLen16 { out },
            "1+1 != 2"
        );

        // check fulladder carry
        let mut num1 = [false; 16];
        num1[14] = true;
        num1[15] = true;
        let mut num2 = [false; 16];
        num2[14] = true;
        num2[15] = true;
        let mut out = [false; 16];
        out[14] = true;
        out[13] = true;
        assert_eq!(
            machine.process(Adder16Inputs { num1, num2 }),
            ArrayLen16 { out },
            "3+3 != 6"
        );

        // check overflow over at MSB
        let num1 = [true; 16];
        let num2 = [true; 16];
        let mut out = [true; 16];
        out[15] = false;
        assert_eq!(
            machine.process(Adder16Inputs { num1, num2 }),
            ArrayLen16 { out },
            "1+1 != 2"
        );

        // check two's complement
        let mut num1 = [true; 16];
        num1[14] = false;
        let mut num2 = [false; 16];
        num2[14] = true;
        let out = [true; 16];
        assert_eq!(
            machine.process(Adder16Inputs { num1, num2 }),
            ArrayLen16 { out },
            "-3+2 != -1"
        );
    }

    #[test]
    fn incrementer16_adds_just_one_to_input() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Incrementer16::from);

        let mut out = [false; 16];
        out[15] = true;
        assert_eq!(
            machine.process(Incrementer16Inputs { num: [false; 16] }),
            ArrayLen16 { out },
            "0+1 != 1"
        );
    }
}

fn main() {
    let alloc = Bump::new();
    let machine = Machine::new(&alloc, Dflipflop::from);
    ui::start_interactive_server(&machine, 3000);
}
