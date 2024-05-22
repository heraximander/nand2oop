use bumpalo::Bump;
use hdl::{ChipInput, ChipOutput, ChipOutputType, Input, Nand, SizedChip};
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
        [Input::ChipInput(input[0]), Input::ChipInput(input[2])],
    );
    let not = Not::new(alloc, [Input::ChipInput(input[2])]);
    let and2 = And::new(
        alloc,
        [
            Input::ChipInput(input[1]),
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
        assert_eq!(machine.process([true, false, true]), [true]);
        assert_eq!(machine.process([false, true, true]), [false]);
        assert_eq!(machine.process([false, false, true]), [false]);
        assert_eq!(machine.process([true, false, false]), [false]);
        assert_eq!(machine.process([false, true, false]), [true]);
        assert_eq!(machine.process([false, false, false]), [false]);
    }
}

fn main() {
    println!("Hello, world!");
}
