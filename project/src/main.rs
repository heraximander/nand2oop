use std::iter;

use bumpalo::Bump;
use hdl::{ChipInput, ChipOutput, ChipOutputType, Input, Nand, SizedChip, UserInput};
use hdl_macro::chip;

#[chip]
fn not<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 1]) -> [ChipOutputType<'a>; 1] {
    let nand = Nand::new(
        &alloc,
        Input::ChipInput(input[0]),
        Input::ChipInput(input[0]),
    );
    [ChipOutputType::NandOutput(nand)]
}

#[chip]
fn and<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 2]) -> [ChipOutputType<'a>; 1] {
    let nand = Nand::new(
        &alloc,
        Input::ChipInput(input[0]),
        Input::ChipInput(input[1]),
    );
    let not = Not::new(alloc, [Input::NandInput(nand)]);
    [ChipOutputType::ChipOutput(not.get_out(alloc)[0])]
}

#[chip]
fn or<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 2]) -> [ChipOutputType<'a>; 1] {
    let not1 = Not::new(&alloc, [Input::ChipInput(input[0])]);
    let not2 = Not::new(&alloc, [Input::ChipInput(input[1])]);
    let nand = Nand::new(
        &alloc,
        Input::ChipOutput(not1.get_out(alloc)[0]),
        Input::ChipOutput(not2.get_out(alloc)[0]),
    );
    [ChipOutputType::NandOutput(nand)]
}

#[chip]
fn xor<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 2]) -> [ChipOutputType<'a>; 1] {
    let and = And::new(
        &alloc,
        [Input::ChipInput(input[0]), Input::ChipInput(input[1])],
    );
    let not = Not::new(&alloc, [Input::ChipOutput(and.get_out(alloc)[0])]);
    let or = Or::new(
        &alloc,
        [Input::ChipInput(input[0]), Input::ChipInput(input[1])],
    );
    let and2 = And::new(
        &alloc,
        [
            Input::ChipOutput(not.get_out(alloc)[0]),
            Input::ChipOutput(or.get_out(alloc)[0]),
        ],
    );
    [ChipOutputType::ChipOutput(and2.get_out(alloc)[0])]
}

#[chip]
fn mux<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 3]) -> [ChipOutputType<'a>; 1] {
    let and1 = And::new(
        alloc,
        [Input::ChipInput(input[1]), Input::ChipInput(input[2])],
    );
    let not = Not::new(alloc, [Input::ChipInput(input[2])]);
    let and2 = And::new(
        alloc,
        [
            Input::ChipInput(input[0]),
            Input::ChipOutput(not.get_out(alloc)[0]),
        ],
    );
    let or = Or::new(
        alloc,
        [
            Input::ChipOutput(and1.get_out(alloc)[0]),
            Input::ChipOutput(and2.get_out(alloc)[0]),
        ],
    );
    [ChipOutputType::ChipOutput(or.get_out(alloc)[0])]
}

#[chip]
fn demux<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 2]) -> [ChipOutputType<'a>; 2] {
    let and1 = And::new(
        alloc,
        [Input::ChipInput(input[0]), Input::ChipInput(input[1])],
    );
    let not = Not::new(alloc, [Input::ChipInput(input[1])]);
    let and2 = And::new(
        alloc,
        [
            Input::ChipInput(input[0]),
            Input::ChipOutput(not.get_out(alloc)[0]),
        ],
    );
    [
        ChipOutputType::ChipOutput(and2.get_out(alloc)[0]),
        ChipOutputType::ChipOutput(and1.get_out(alloc)[0]),
    ]
}

#[chip]
fn not2<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 2]) -> [ChipOutputType<'a>; 2] {
    // TODO: note that we can generalise this function to `NOT _n_`
    input.map(|in_| {
        ChipOutputType::ChipOutput(Not::new(alloc, [Input::ChipInput(in_)]).get_out(alloc)[0])
    })
}

#[chip]
fn and2<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 4]) -> [ChipOutputType<'a>; 2] {
    /* writing out this zip is painful, but if we slice `input` we lose the size information
       required by the return type
       we could:
       1. rewrite this to use a for... loop. Probably the most sensible option.
       2. write a macro for slicing a known-size array in to smaller arrays. Note that this will
          result in copying.
       3. Slice `input` and `.collect()` as `Vec` at the end before `.try_into()`ing in to an array.
          Easiest way to continue using iterators but looks gross.
    */
    [(input[0], input[2]), (input[1], input[3])].map(|(in1, in2)| {
        ChipOutputType::ChipOutput(
            And::new(alloc, [Input::ChipInput(in1), Input::ChipInput(in2)]).get_out(alloc)[0],
        )
    })
}

#[chip]
fn or2<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 4]) -> [ChipOutputType<'a>; 2] {
    [(input[0], input[2]), (input[1], input[3])].map(|(in1, in2)| {
        ChipOutputType::ChipOutput(
            Or::new(alloc, [Input::ChipInput(in1), Input::ChipInput(in2)]).get_out(alloc)[0],
        )
    })
}

#[chip]
fn mux2<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 5]) -> [ChipOutputType<'a>; 2] {
    [(input[0], input[2]), (input[1], input[3])].map(|(in1, in2)| {
        ChipOutputType::ChipOutput(
            Mux::new(
                alloc,
                [
                    Input::ChipInput(in1),
                    Input::ChipInput(in2),
                    Input::ChipInput(input[4]),
                ],
            )
            .get_out(alloc)[0],
        )
    })
}

#[chip]
fn andmult4<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 4]) -> [ChipOutputType<'a>; 1] {
    let initial_and = And::new(
        alloc,
        [Input::ChipInput(input[0]), Input::ChipInput(input[1])],
    )
    .get_out(alloc)[0];
    let final_output = input.iter().skip(2).fold(initial_and, |acc, in_| {
        And::new(alloc, [Input::ChipInput(in_), Input::ChipOutput(acc)]).get_out(alloc)[0]
    });
    [ChipOutputType::ChipOutput(final_output)]
}

#[chip]
fn halfadder<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 2]) -> [ChipOutputType<'a>; 2] {
    let sum_bit = Xor::new(
        alloc,
        [Input::ChipInput(input[0]), Input::ChipInput(input[1])],
    );
    let carry_bit = And::new(
        alloc,
        [Input::ChipInput(input[0]), Input::ChipInput(input[1])],
    );
    [
        ChipOutputType::ChipOutput(carry_bit.get_out(alloc)[0]),
        ChipOutputType::ChipOutput(sum_bit.get_out(alloc)[0]),
    ]
}

#[chip]
fn fulladder<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 3]) -> [ChipOutputType<'a>; 2] {
    let first_hadder = Halfadder::new(
        alloc,
        [Input::ChipInput(input[0]), Input::ChipInput(input[1])],
    );
    let second_hadder = Halfadder::new(
        alloc,
        [
            Input::ChipInput(input[2]),
            Input::ChipOutput(first_hadder.get_out(alloc)[1]),
        ],
    );
    let carry_or = Or::new(
        alloc,
        [
            Input::ChipOutput(first_hadder.get_out(alloc)[0]),
            Input::ChipOutput(second_hadder.get_out(alloc)[0]),
        ],
    );
    [
        ChipOutputType::ChipOutput(carry_or.get_out(alloc)[0]),
        ChipOutputType::ChipOutput(second_hadder.get_out(alloc)[1]),
    ]
}

#[chip]
fn adder16<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 32]) -> [ChipOutputType<'a>; 16] {
    let fnum = &input[..16];
    let snum = &input[16..];

    let lsb = Halfadder::new(
        alloc,
        [Input::ChipInput(fnum[15]), Input::ChipInput(snum[15])],
    );
    let zipin = fnum[..15]
        .iter()
        .zip(&snum[..15])
        .rev()
        .fold(vec![lsb.get_out(alloc)], |mut acc, x| {
            let prev_carry = acc.last().unwrap()[0];
            let adder = Fulladder::new(
                alloc,
                [
                    Input::ChipOutput(prev_carry),
                    Input::ChipInput(x.0),
                    Input::ChipInput(x.1),
                ],
            );
            acc.push(adder.get_out(alloc));
            acc
        })
        .iter()
        .map(|out| ChipOutputType::ChipOutput(out[1]))
        .rev()
        .collect::<Vec<_>>();

    zipin
        .try_into()
        .unwrap_or_else(|_| panic!("output must be exactly half of input"))
}

#[chip]
fn incrementer16<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 16]) -> [ChipOutputType<'a>; 16] {
    let inputs = input.map(|in_| Input::ChipInput(in_));
    let adder_inputs = iter::repeat_with(|| Input::UserInput(UserInput::from(alloc, false)))
        .take(15)
        .chain(iter::once(Input::UserInput(UserInput::from(alloc, true))))
        .chain(inputs)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap_or_else(|_| panic!("array must be length 16"));
    let adder = Adder16::new(alloc, adder_inputs);
    adder.get_out(alloc).map(|x| ChipOutputType::ChipOutput(x))
}

#[cfg(test)]
mod tests {
    use bumpalo::Bump;
    use hdl::Machine;

    use crate::*;

    #[test]
    fn not_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Not::new);
        assert_eq!(machine.process([true]), [false]);
        assert_eq!(machine.process([false]), [true]);
    }

    #[test]
    fn and_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, And::new);
        assert_eq!(machine.process([true, true]), [true]);
        assert_eq!(machine.process([true, false]), [false]);
        assert_eq!(machine.process([false, true]), [false]);
        assert_eq!(machine.process([false, false]), [false]);
    }

    #[test]
    fn or_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Or::new);
        assert_eq!(machine.process([true, true]), [true]);
        assert_eq!(machine.process([true, false]), [true]);
        assert_eq!(machine.process([false, true]), [true]);
        assert_eq!(machine.process([false, false]), [false]);
    }

    #[test]
    fn xor_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Xor::new);
        assert_eq!(machine.process([true, true]), [false]);
        assert_eq!(machine.process([true, false]), [true]);
        assert_eq!(machine.process([false, true]), [true]);
        assert_eq!(machine.process([false, false]), [false]);
    }

    #[test]
    fn mux_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Mux::new);
        assert_eq!(machine.process([true, true, true]), [true]);
        assert_eq!(machine.process([true, true, false]), [true]);
        assert_eq!(machine.process([true, false, true]), [false]);
        assert_eq!(machine.process([false, true, true]), [true]);
        assert_eq!(machine.process([false, false, true]), [false]);
        assert_eq!(machine.process([true, false, false]), [true]);
        assert_eq!(machine.process([false, true, false]), [false]);
        assert_eq!(machine.process([false, false, false]), [false]);
    }

    #[test]
    fn demux_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Demux::new);
        assert_eq!(machine.process([true, true]), [false, true]);
        assert_eq!(machine.process([true, false]), [true, false]);
        assert_eq!(machine.process([false, true]), [false, false]);
        assert_eq!(machine.process([false, false]), [false, false]);
    }

    #[test]
    fn not2_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Not2::new);
        assert_eq!(machine.process([true, true]), [false, false]);
        assert_eq!(machine.process([true, false]), [false, true]);
        assert_eq!(machine.process([false, true]), [true, false]);
        assert_eq!(machine.process([false, false]), [true, true]);
    }

    #[test]
    fn and2_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, And2::new);
        assert_eq!(machine.process([true, true, true, true]), [true, true]);
        assert_eq!(machine.process([false, true, true, true]), [false, true]);
        assert_eq!(machine.process([true, false, true, true]), [true, false]);
        assert_eq!(machine.process([true, true, false, true]), [false, true]);
        assert_eq!(machine.process([true, true, true, false]), [true, false]);
        // ...
        assert_eq!(
            machine.process([false, false, false, false]),
            [false, false]
        );
    }

    #[test]
    fn or2_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Or2::new);
        assert_eq!(machine.process([true, true, true, true]), [true, true]);
        assert_eq!(machine.process([false, true, true, true]), [true, true]);
        assert_eq!(machine.process([true, false, true, true]), [true, true]);
        assert_eq!(machine.process([true, true, false, true]), [true, true]);
        assert_eq!(machine.process([true, true, true, false]), [true, true]);
        // ...
        assert_eq!(machine.process([false, true, false, false]), [false, true]);
        assert_eq!(
            machine.process([false, false, false, false]),
            [false, false]
        );
    }

    #[test]
    fn mux2_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Mux2::new);
        assert_eq!(
            machine.process([true, true, false, true, true]),
            [false, true]
        );
        assert_eq!(
            machine.process([true, true, false, true, false]),
            [true, true]
        );
        // ...
    }

    #[test]
    fn andmult4_gate_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Andmult4::new);
        assert_eq!(machine.process([true, true, true, true]), [true]);
        assert_eq!(machine.process([false, true, true, true]), [false]);
        assert_eq!(machine.process([true, false, true, true]), [false]);
        assert_eq!(machine.process([true, true, false, true]), [false]);
        assert_eq!(machine.process([true, true, true, false]), [false]);
        // ...
    }

    #[test]
    fn halfadder_chip_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Halfadder::new);
        assert_eq!(machine.process([false, false]), [false, false]);
        assert_eq!(machine.process([false, true]), [false, true]);
        assert_eq!(machine.process([true, false]), [false, true]);
        assert_eq!(machine.process([true, true]), [true, false]);
    }

    #[test]
    fn fulladder_chip_has_correct_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Fulladder::new);

        assert_eq!(machine.process([false, false, false]), [false, false]);
        assert_eq!(machine.process([false, false, true]), [false, true]);
        assert_eq!(machine.process([false, true, false]), [false, true]);
        assert_eq!(machine.process([true, false, false]), [false, true]);
        assert_eq!(machine.process([false, true, true]), [true, false]);
        assert_eq!(machine.process([true, false, true]), [true, false]);
        assert_eq!(machine.process([true, true, false]), [true, false]);
        assert_eq!(machine.process([true, true, true]), [true, true]);
    }

    #[test]
    fn adder16_chip_has_correct_partial_truth_table() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Adder16::new);

        assert_eq!(machine.process([false; 32]), [false; 16], "0+0 != 0");

        // check LSB and MSB values are represented
        let mut in_ = [false; 32];
        let in1 = &mut in_[..16];
        in1[15] = true;
        let in2 = &mut in_[16..];
        in2[0] = true;
        let mut out = [false; 16];
        out[0] = true;
        out[15] = true;
        assert_eq!(machine.process(in_), out, "1+32768 != 32769");

        // check halfadder carry
        let mut in_ = [false; 32];
        let in1 = &mut in_[..16];
        in1[15] = true;
        let in2 = &mut in_[16..];
        in2[15] = true;
        let mut out = [false; 16];
        out[15 - 1] = true;
        assert_eq!(machine.process(in_), out, "1+1 != 2");

        // check fulladder carry
        let mut in_ = [false; 32];
        let in1 = &mut in_[..16];
        in1[14] = true;
        in1[15] = true;
        let in2 = &mut in_[16..];
        in2[14] = true;
        in2[15] = true;
        let mut out = [false; 16];
        out[14] = true;
        out[13] = true;
        assert_eq!(machine.process(in_), out, "3+3 != 6");

        // check overflow over at MSB
        let in_ = [true; 32];
        let mut out = [true; 16];
        out[15] = false;
        assert_eq!(machine.process(in_), out, "1+1 != 2");

        // check two's complement
        let mut in_ = [true; 32];
        let in1 = &mut in_[..16];
        in1[14] = false;
        let in2 = &mut in_[16..];
        for i in 0..16 {
            in2[i] = false;
        }
        in2[14] = true;
        let out = [true; 16];
        assert_eq!(machine.process(in_), out, "-3+2 != -1");
    }

    #[test]
    fn incrementer16_adds_just_one_to_input() {
        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Incrementer16::new);

        let mut out = [false; 16];
        out[15] = true;
        assert_eq!(machine.process([false; 16]), out, "0+1 != 1");
    }
}

fn main() {
    println!("Hello, world!");
}

// TODO, ideally after I inplement the ALU but if it's painful doing the ALU without them, then well...:
// 1. Add masks for inputs and outputs, to give them names
// 2. Add more legible diagramming:
//    1. At least add the names for inputs and outputs to the diagrams...
//    2. Add an interactive version which hides the chip details until the user clicks on the chip
// 3. Add muxn chip? Using generic constants to decide how large the outputs are
// 4. Rename and refactor UserInput to be a general set-value Input
