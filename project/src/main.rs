use std::{
    array::{self, from_fn},
    iter,
};

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

#[derive(StructuredData, PartialEq, Debug)]
struct BinaryArrayLen16<T> {
    out1: [T; 16],
    out2: [T; 16],
}

#[derive(StructuredData, PartialEq, Debug)]
struct OctArrayLen16<T> {
    out1: [T; 16],
    out2: [T; 16],
    out3: [T; 16],
    out4: [T; 16],
    out5: [T; 16],
    out6: [T; 16],
    out7: [T; 16],
    out8: [T; 16],
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
fn demux16<'a>(
    alloc: &'a Bump,
    in_: [&'a ChipInput<'a>; 16],
    sel: &'a ChipInput<'a>,
) -> BinaryArrayLen16<ChipOutputType<'a>> {
    let out = in_.map(|elem| Demux::new(alloc, elem.into(), sel.into()).get_out(alloc));
    let out1 = from_fn(|i| out[i].out1.into());
    let out2 = from_fn(|i| out[i].out2.into());
    BinaryArrayLen16 { out1, out2 }
}

#[chip]
fn demux16x8<'a>(
    alloc: &'a Bump,
    in_: [&'a ChipInput<'a>; 16],
    sel: [&'a ChipInput<'a>; 3],
) -> OctArrayLen16<ChipOutputType<'a>> {
    let demux1 = Demux16::new(alloc, in_.ainto(), sel[0].into());
    let dmx1o = demux1.get_out(alloc);

    let demux2 = Demux16::new(alloc, dmx1o.out1.ainto(), sel[1].into());
    let demux3 = Demux16::new(alloc, dmx1o.out2.ainto(), sel[1].into());
    let dmx2o = demux2.get_out(alloc);
    let dmx3o = demux3.get_out(alloc);

    let demux4 = Demux16::new(alloc, dmx2o.out1.ainto(), sel[2].into());
    let demux5 = Demux16::new(alloc, dmx2o.out2.ainto(), sel[2].into());
    let demux6 = Demux16::new(alloc, dmx3o.out1.ainto(), sel[2].into());
    let demux7 = Demux16::new(alloc, dmx3o.out2.ainto(), sel[2].into());
    let dmx4o = demux4.get_out(alloc);
    let dmx5o = demux5.get_out(alloc);
    let dmx6o = demux6.get_out(alloc);
    let dmx7o = demux7.get_out(alloc);

    OctArrayLen16 {
        out1: dmx4o.out1.ainto(),
        out2: dmx4o.out2.ainto(),
        out3: dmx5o.out1.ainto(),
        out4: dmx5o.out2.ainto(),
        out5: dmx6o.out1.ainto(),
        out6: dmx6o.out2.ainto(),
        out7: dmx7o.out1.ainto(),
        out8: dmx7o.out2.ainto(),
    }
}

#[chip]
fn mux16x8<'a>(
    alloc: &'a Bump,
    in1: [&'a ChipInput<'a>; 16],
    in2: [&'a ChipInput<'a>; 16],
    in3: [&'a ChipInput<'a>; 16],
    in4: [&'a ChipInput<'a>; 16],
    in5: [&'a ChipInput<'a>; 16],
    in6: [&'a ChipInput<'a>; 16],
    in7: [&'a ChipInput<'a>; 16],
    in8: [&'a ChipInput<'a>; 16],
    sel: [&'a ChipInput<'a>; 3],
) -> ArrayLen16<ChipOutputType<'a>> {
    let mux1 = Mux16::new(alloc, in1.ainto(), in2.ainto(), sel[2].into());
    let mux2 = Mux16::new(alloc, in3.ainto(), in4.ainto(), sel[2].into());
    let mux3 = Mux16::new(alloc, in5.ainto(), in6.ainto(), sel[2].into());
    let mux4 = Mux16::new(alloc, in7.ainto(), in8.ainto(), sel[2].into());

    let mux5 = Mux16::new(
        alloc,
        mux1.get_out(alloc).out.ainto(),
        mux2.get_out(alloc).out.ainto(),
        sel[1].into(),
    );
    let mux6 = Mux16::new(
        alloc,
        mux3.get_out(alloc).out.ainto(),
        mux4.get_out(alloc).out.ainto(),
        sel[1].into(),
    );

    let mux7 = Mux16::new(
        alloc,
        mux5.get_out(alloc).out.ainto(),
        mux6.get_out(alloc).out.ainto(),
        sel[0].into(),
    );

    ArrayLen16 {
        out: mux7.get_out(alloc).out.ainto(),
    }
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

#[chip]
fn ram8<'a>(
    alloc: &'a Bump,
    in_: [&'a ChipInput<'a>; 16],
    address: [&'a ChipInput<'a>; 3],
    load: &'a ChipInput<'a>,
    clock: &'a ChipInput<'a>,
) -> ArrayLen16<ChipOutputType<'a>> {
    let demux = Demux1x8::new(alloc, load.into(), address.ainto());
    let dmxo = demux.get_out(alloc);

    let reg1 = Register16::new(alloc, in_.ainto(), dmxo.out1.into(), clock.into());
    let reg2 = Register16::new(alloc, in_.ainto(), dmxo.out2.into(), clock.into());
    let reg3 = Register16::new(alloc, in_.ainto(), dmxo.out3.into(), clock.into());
    let reg4 = Register16::new(alloc, in_.ainto(), dmxo.out4.into(), clock.into());
    let reg5 = Register16::new(alloc, in_.ainto(), dmxo.out5.into(), clock.into());
    let reg6 = Register16::new(alloc, in_.ainto(), dmxo.out6.into(), clock.into());
    let reg7 = Register16::new(alloc, in_.ainto(), dmxo.out7.into(), clock.into());
    let reg8 = Register16::new(alloc, in_.ainto(), dmxo.out8.into(), clock.into());

    let mux = Mux16x8::new(
        alloc,
        reg1.get_out(alloc).out.ainto(),
        reg2.get_out(alloc).out.ainto(),
        reg3.get_out(alloc).out.ainto(),
        reg4.get_out(alloc).out.ainto(),
        reg5.get_out(alloc).out.ainto(),
        reg6.get_out(alloc).out.ainto(),
        reg7.get_out(alloc).out.ainto(),
        reg8.get_out(alloc).out.ainto(),
        address.ainto(),
    );

    ArrayLen16 {
        out: mux.get_out(alloc).out.ainto(),
    }
}

#[chip]
fn ram64<'a>(
    alloc: &'a Bump,
    in_: [&'a ChipInput<'a>; 16],
    address: [&'a ChipInput<'a>; 6],
    load: &'a ChipInput<'a>,
    clock: &'a ChipInput<'a>,
) -> ArrayLen16<ChipOutputType<'a>> {
    let this_addr = from_fn(|i| address[i]);
    let remaining_addr = from_fn(|i| address[i + 3]);
    let demux = Demux1x8::new(alloc, load.into(), this_addr.ainto());
    let dmxo = demux.get_out(alloc);

    let reg1 = Ram8::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out1.into(),
        clock.into(),
    );
    let reg2 = Ram8::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out2.into(),
        clock.into(),
    );
    let reg3 = Ram8::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out3.into(),
        clock.into(),
    );
    let reg4 = Ram8::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out4.into(),
        clock.into(),
    );
    let reg5 = Ram8::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out5.into(),
        clock.into(),
    );
    let reg6 = Ram8::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out6.into(),
        clock.into(),
    );
    let reg7 = Ram8::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out7.into(),
        clock.into(),
    );
    let reg8 = Ram8::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out8.into(),
        clock.into(),
    );

    let mux = Mux16x8::new(
        alloc,
        reg1.get_out(alloc).out.ainto(),
        reg2.get_out(alloc).out.ainto(),
        reg3.get_out(alloc).out.ainto(),
        reg4.get_out(alloc).out.ainto(),
        reg5.get_out(alloc).out.ainto(),
        reg6.get_out(alloc).out.ainto(),
        reg7.get_out(alloc).out.ainto(),
        reg8.get_out(alloc).out.ainto(),
        this_addr.ainto(),
    );

    ArrayLen16 {
        out: mux.get_out(alloc).out.ainto(),
    }
}

#[chip]
fn ram512<'a>(
    alloc: &'a Bump,
    in_: [&'a ChipInput<'a>; 16],
    address: [&'a ChipInput<'a>; 9],
    load: &'a ChipInput<'a>,
    clock: &'a ChipInput<'a>,
) -> ArrayLen16<ChipOutputType<'a>> {
    let this_addr = from_fn(|i| address[i]);
    let remaining_addr = from_fn(|i| address[i + 3]);
    let demux = Demux1x8::new(alloc, load.into(), this_addr.ainto());
    let dmxo = demux.get_out(alloc);

    let reg1 = Ram64::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out1.into(),
        clock.into(),
    );
    let reg2 = Ram64::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out2.into(),
        clock.into(),
    );
    let reg3 = Ram64::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out3.into(),
        clock.into(),
    );
    let reg4 = Ram64::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out4.into(),
        clock.into(),
    );
    let reg5 = Ram64::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out5.into(),
        clock.into(),
    );
    let reg6 = Ram64::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out6.into(),
        clock.into(),
    );
    let reg7 = Ram64::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out7.into(),
        clock.into(),
    );
    let reg8 = Ram64::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out8.into(),
        clock.into(),
    );

    let mux = Mux16x8::new(
        alloc,
        reg1.get_out(alloc).out.ainto(),
        reg2.get_out(alloc).out.ainto(),
        reg3.get_out(alloc).out.ainto(),
        reg4.get_out(alloc).out.ainto(),
        reg5.get_out(alloc).out.ainto(),
        reg6.get_out(alloc).out.ainto(),
        reg7.get_out(alloc).out.ainto(),
        reg8.get_out(alloc).out.ainto(),
        this_addr.ainto(),
    );

    ArrayLen16 {
        out: mux.get_out(alloc).out.ainto(),
    }
}

#[chip]
fn ram16k<'a>(
    alloc: &'a Bump,
    in_: [&'a ChipInput<'a>; 16],
    address: [&'a ChipInput<'a>; 12],
    load: &'a ChipInput<'a>,
    clock: &'a ChipInput<'a>,
) -> ArrayLen16<ChipOutputType<'a>> {
    let this_addr = from_fn(|i| address[i]);
    let remaining_addr = from_fn(|i| address[i + 3]);
    let demux = Demux1x4::new(alloc, load.into(), this_addr.ainto());
    let dmxo = demux.get_out(alloc);

    let reg1 = Ram512::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out1.into(),
        clock.into(),
    );
    let reg2 = Ram512::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out2.into(),
        clock.into(),
    );
    let reg3 = Ram512::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out3.into(),
        clock.into(),
    );
    let reg4 = Ram512::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out4.into(),
        clock.into(),
    );

    let mux = Mux16x4::new(
        alloc,
        reg1.get_out(alloc).out.ainto(),
        reg2.get_out(alloc).out.ainto(),
        reg3.get_out(alloc).out.ainto(),
        reg4.get_out(alloc).out.ainto(),
        this_addr.ainto(),
    );

    ArrayLen16 {
        out: mux.get_out(alloc).out.ainto(),
    }
}

#[chip]
fn ram4k<'a>(
    alloc: &'a Bump,
    in_: [&'a ChipInput<'a>; 16],
    address: [&'a ChipInput<'a>; 12],
    load: &'a ChipInput<'a>,
    clock: &'a ChipInput<'a>,
) -> ArrayLen16<ChipOutputType<'a>> {
    let this_addr = from_fn(|i| address[i]);
    let remaining_addr = from_fn(|i| address[i + 3]);
    let demux = Demux1x8::new(alloc, load.into(), this_addr.ainto());
    let dmxo = demux.get_out(alloc);

    let reg1 = Ram512::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out1.into(),
        clock.into(),
    );
    let reg2 = Ram512::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out2.into(),
        clock.into(),
    );
    let reg3 = Ram512::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out3.into(),
        clock.into(),
    );
    let reg4 = Ram512::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out4.into(),
        clock.into(),
    );
    let reg5 = Ram512::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out5.into(),
        clock.into(),
    );
    let reg6 = Ram512::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out6.into(),
        clock.into(),
    );
    let reg7 = Ram512::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out7.into(),
        clock.into(),
    );
    let reg8 = Ram512::new(
        alloc,
        in_.ainto(),
        remaining_addr.ainto(),
        dmxo.out8.into(),
        clock.into(),
    );

    let mux = Mux16x8::new(
        alloc,
        reg1.get_out(alloc).out.ainto(),
        reg2.get_out(alloc).out.ainto(),
        reg3.get_out(alloc).out.ainto(),
        reg4.get_out(alloc).out.ainto(),
        reg5.get_out(alloc).out.ainto(),
        reg6.get_out(alloc).out.ainto(),
        reg7.get_out(alloc).out.ainto(),
        reg8.get_out(alloc).out.ainto(),
        this_addr.ainto(),
    );

    ArrayLen16 {
        out: mux.get_out(alloc).out.ainto(),
    }
}

#[cfg(test)]
mod tests {
    use std::{i16, usize};

    use crate::*;
    use bumpalo::Bump;
    use hdl::Machine;

    fn ntb<const N: usize>(in_: i16) -> [bool; N] {
        let in32 = i32::from(in_);
        let mut ret = from_fn(|i| {
            let mask = (2 as i32).pow(i as u32);
            if in32 & mask == mask {
                true
            } else {
                false
            }
        });
        ret.reverse();
        ret
    }

    #[test]
    fn number_to_bool_array_works_as_expected() {
        let num = ntb(5);
        assert_eq!(num, [true, false, true]);
    }

    #[test]
    fn register16_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Register16::from);
        let res = machine.process(Register16Inputs {
            in_: ntb(4321),
            load: true,
            clock: true,
        }); // initial state
        assert_eq!(res.out, ntb(0));
        let res = machine.process(Register16Inputs {
            in_: ntb(0),
            load: true,
            clock: false,
        }); // tock
        assert_eq!(res.out, ntb(4321));
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
            x: ntb(1),
            y: ntb(1),
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
                out: ntb(2),
                zr: false,
                ng: false
            }
        );

        // zx works
        let res = machine.process(AluInputs {
            x: ntb(1),
            y: ntb(2),
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
                out: ntb(2),
                zr: false,
                ng: false
            }
        );

        // zy works
        let res = machine.process(AluInputs {
            x: ntb(1),
            y: ntb(2),
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
                out: ntb(1),
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
                out: ntb(-2),
                zr: false,
                ng: true
            }
        );

        // ny works
        let res = machine.process(AluInputs {
            x: ntb(-1),
            y: ntb(0),
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
                out: ntb(-2),
                zr: false,
                ng: true
            }
        );

        // no works
        let res = machine.process(AluInputs {
            x: ntb(-1),
            y: ntb(-1),
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
                out: ntb(1),
                ng: false,
                zr: false
            }
        );

        // and works
        let res = machine.process(AluInputs {
            x: ntb(-1),
            y: ntb(-1),
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
                out: ntb(-1),
                ng: true,
                zr: false
            }
        );

        // now I'll just put in the rest of the truth table as per the book
        // 0
        let res = machine.process(AluInputs {
            x: ntb(-1),
            y: ntb(-1),
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
                out: ntb(0),
                ng: false,
                zr: true
            }
        );

        // 1
        let res = machine.process(AluInputs {
            x: ntb(-1),
            y: ntb(-1),
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
                out: ntb(1),
                ng: false,
                zr: false
            }
        );

        // -1
        let res = machine.process(AluInputs {
            x: ntb(132),
            y: ntb(876),
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
                out: ntb(-1),
                ng: true,
                zr: false
            }
        );

        // x
        let res = machine.process(AluInputs {
            x: ntb(452),
            y: ntb(671),
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
                out: ntb(452),
                ng: false,
                zr: false
            }
        );

        // y
        let res = machine.process(AluInputs {
            x: ntb(452),
            y: ntb(671),
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
                out: ntb(671),
                ng: false,
                zr: false
            }
        );

        // !x
        let res = machine.process(AluInputs {
            x: ntb(452),
            y: ntb(671),
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
                out: ntb(!452),
                ng: true,
                zr: false
            }
        );

        // !y
        let res = machine.process(AluInputs {
            x: ntb(452),
            y: ntb(671),
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
                out: ntb(!671),
                ng: true,
                zr: false
            }
        );

        // x+1
        let res = machine.process(AluInputs {
            x: ntb(452),
            y: ntb(671),
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
                out: ntb(452 + 1),
                ng: false,
                zr: false
            }
        );

        // y+1
        let res = machine.process(AluInputs {
            x: ntb(452),
            y: ntb(671),
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
                out: ntb(671 + 1),
                ng: false,
                zr: false
            }
        );

        // x-1
        let res = machine.process(AluInputs {
            x: ntb(452),
            y: ntb(671),
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
                out: ntb(452 - 1),
                ng: false,
                zr: false
            }
        );

        // y-1
        let res = machine.process(AluInputs {
            x: ntb(452),
            y: ntb(671),
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
                out: ntb(671 - 1),
                ng: false,
                zr: false
            }
        );

        // x+y
        let res = machine.process(AluInputs {
            x: ntb(452),
            y: ntb(671),
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
                out: ntb(452 + 671),
                ng: false,
                zr: false
            }
        );

        // x-y
        let res = machine.process(AluInputs {
            x: ntb(452),
            y: ntb(671),
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
                out: ntb(452 - 671),
                ng: true,
                zr: false
            }
        );

        // y-x
        let res = machine.process(AluInputs {
            x: ntb(452),
            y: ntb(671),
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
                out: ntb(671 - 452),
                ng: false,
                zr: false
            }
        );

        // x|y
        let res = machine.process(AluInputs {
            x: ntb(452),
            y: ntb(671),
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
                out: ntb(452 | 671),
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
    fn and16_gate_has_correct_truth_table() {
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
                in1: ntb(i16::MAX),
                in2: ntb(-1)
            }),
            ArrayLen16 { out: ntb(i16::MAX) }
        );
        assert_eq!(
            machine.process(And16Inputs {
                in1: ntb(-5),
                in2: ntb(-1)
            }),
            ArrayLen16 { out: ntb(-5) }
        );
        assert_eq!(
            machine.process(And16Inputs {
                in1: ntb(-1),
                in2: ntb(i16::MAX)
            }),
            ArrayLen16 { out: ntb(i16::MAX) }
        );
        assert_eq!(
            machine.process(And16Inputs {
                in1: ntb(-1),
                in2: ntb(-765)
            }),
            ArrayLen16 { out: ntb(-765) }
        );
        // ...
        assert_eq!(
            machine.process(And16Inputs {
                in1: ntb(0),
                in2: ntb(0)
            }),
            ArrayLen16 { out: ntb(0) }
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
    fn demux16_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Demux16::from);
        assert_eq!(
            machine.process(Demux16Inputs {
                in_: [true; 16],
                sel: true
            }),
            BinaryArrayLen16 {
                out1: [false; 16],
                out2: [true; 16]
            }
        );
        assert_eq!(
            machine.process(Demux16Inputs {
                in_: [true; 16],
                sel: false
            }),
            BinaryArrayLen16 {
                out1: [true; 16],
                out2: [false; 16]
            }
        );
        // ...
    }

    #[test]
    fn mux16x8_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Mux16x8::from);
        let out = machine.process(Mux16x8Inputs {
            in1: [true; 16],
            in2: [false; 16],
            in3: [false; 16],
            in4: [false; 16],
            in5: [false; 16],
            in6: [false; 16],
            in7: [false; 16],
            in8: [false; 16],
            sel: [false, false, false],
        });
        assert_eq!(out.out, [true; 16]);

        let out = machine.process(Mux16x8Inputs {
            in1: [true; 16],
            in2: [false; 16],
            in3: [false; 16],
            in4: [false; 16],
            in5: [false; 16],
            in6: [false; 16],
            in7: [false; 16],
            in8: [false; 16],
            sel: [true, true, true],
        });
        assert_eq!(out.out, [false; 16]);

        let out = machine.process(Mux16x8Inputs {
            in1: [false; 16],
            in2: [false; 16],
            in3: [false; 16],
            in4: [false; 16],
            in5: [true; 16],
            in6: [false; 16],
            in7: [false; 16],
            in8: [false; 16],
            sel: [true, false, false],
        });
        assert_eq!(out.out, [true; 16]);

        let out = machine.process(Mux16x8Inputs {
            in1: [false; 16],
            in2: [false; 16],
            in3: [false; 16],
            in4: [false; 16],
            in5: [true; 16],
            in6: [false; 16],
            in7: [false; 16],
            in8: [false; 16],
            sel: [true, true, false],
        });
        assert_eq!(out.out, [false; 16]);

        // ...
    }

    #[test]
    fn demux16x8_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Demux16x8::from);
        let out = machine.process(Demux16x8Inputs {
            in_: [true; 16],
            sel: [true, true, true],
        });
        assert_eq!(out.out8, [true; 16]);
        assert_eq!(out.out7, [false; 16]);
        assert_eq!(out.out1, [false; 16]);

        let out = machine.process(Demux16x8Inputs {
            in_: [true; 16],
            sel: [false, true, true],
        });

        assert_eq!(out.out4, [true; 16]);
        assert_eq!(out.out8, [false; 16]);
        assert_eq!(out.out3, [false; 16]);

        let out = machine.process(Demux16x8Inputs {
            in_: [true; 16],
            sel: [false, false, false],
        });

        assert_eq!(out.out1, [true; 16]);
        assert_eq!(out.out8, [false; 16]);
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
                num1: ntb(0),
                num2: ntb(0)
            }),
            ArrayLen16 { out: ntb(0) }
        );

        // check LSB and MSB values are represented
        assert_eq!(
            machine.process(Adder16Inputs {
                num1: ntb(1),
                num2: ntb(-i16::MAX)
            }),
            ArrayLen16 {
                out: ntb(-i16::MAX + 1)
            }
        );

        // check halfadder carry
        assert_eq!(
            machine.process(Adder16Inputs {
                num1: ntb(1),
                num2: ntb(1)
            }),
            ArrayLen16 { out: ntb(2) }
        );

        // check fulladder carry
        assert_eq!(
            machine.process(Adder16Inputs {
                num1: ntb(3),
                num2: ntb(3)
            }),
            ArrayLen16 { out: ntb(6) }
        );

        // check overflow over at MSB
        assert_eq!(
            machine.process(Adder16Inputs {
                num1: ntb(-1),
                num2: ntb(1)
            }),
            ArrayLen16 { out: ntb(0) }
        );
    }

    #[test]
    fn incrementer16_adds_just_one_to_input() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Incrementer16::from);

        assert_eq!(
            machine.process(Incrementer16Inputs { num: ntb(1) }),
            ArrayLen16 { out: ntb(2) }
        );
    }
}

fn main() {
    let alloc = Bump::new();
    let machine = Machine::new(&alloc, Dflipflop::from);
    ui::start_interactive_server(&machine, 3000);
}
