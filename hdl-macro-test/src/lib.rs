#[cfg(test)]
mod tests {
    use bumpalo::Bump;
    use hdl::{ChipInput, ChipOutput, ChipOutputType, Input, Machine, Nand, SizedChip};
    use hdl_macro::chip;

    #[test]
    fn when_a_chip_is_defined_it_can_be_processed_via_machine() {
        #[chip]
        fn testchip<'a>(alloc: &'a Bump, in1: &'a ChipInput<'a>, in2: &'a ChipInput<'a>) -> [ChipOutputType<'a>; 1] {
            let nand = Nand::new(
                &alloc,
                Input::ChipInput(in1),
                Input::ChipInput(in2),
            );
            [ChipOutputType::NandOutput(nand)]
        }

        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Testchip::new);
        assert_eq!(machine.process(TestchipInputs{ in1: true, in2: false }), [true]);
        assert_eq!(machine.process(TestchipInputs{ in1: true, in2: true }), [false]);
    }

    #[test]
    fn when_a_chip_is_defined_with_vector_inputs_it_can_be_processed_via_machine() {
        #[chip]
        fn testchip<'a>(alloc: &'a Bump, num1: [&'a ChipInput<'a>; 2], num2: [&'a ChipInput<'a>; 2], bit: &'a ChipInput<'a>) -> [ChipOutputType<'a>; 2] {
            let bitwise_nand = [
                Nand::new(alloc, Input::ChipInput(num1[0]), Input::ChipInput(num2[0])),
                Nand::new(alloc, Input::ChipInput(num1[1]), Input::ChipInput(num2[1]))
            ];
            bitwise_nand.map(|nand| ChipOutputType::NandOutput(Nand::new(alloc, Input::ChipInput(bit), Input::NandInput(nand))))
        }

        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Testchip::new);
        assert_eq!(machine.process(TestchipInputs { num1: [true, true], num2: [true, false], bit: true }), [true, false]);
        assert_eq!(machine.process(TestchipInputs { num1: [true, true], num2: [true, false], bit: false }), [true, true]);
    }

    #[test]
    fn when_a_nested_chip_is_defined_it_can_be_processed_via_machine() {
        #[chip]
        fn testchip<'a>(alloc: &'a Bump, in1: &'a ChipInput<'a>, in2: &'a ChipInput<'a>) -> [ChipOutputType<'a>; 1] {
            let nand = Nand::new(
                &alloc,
                Input::ChipInput(in1),
                Input::ChipInput(in2),
            );
            [ChipOutputType::NandOutput(nand)]
        }

        #[chip]
        fn testchip2<'a>(
            alloc: &'a Bump,
            in1: &'a ChipInput<'a>, in2: &'a ChipInput<'a>,
        ) -> [ChipOutputType<'a>; 2] {
            let chip = Testchip::new(
                alloc,
                TestchipInputs {
                    in1: Input::ChipInput(in1), 
                    in2: Input::ChipInput(in2),
                }
            );
            let nand = Nand::new(
                &alloc,
                Input::ChipInput(in1),
                Input::ChipOutput(chip.get_out(alloc)[0]),
            );
            [
                ChipOutputType::NandOutput(nand),
                ChipOutputType::ChipInput(in2),
            ]
        }

        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Testchip2::new);
        assert_eq!(machine.process(Testchip2Inputs { in1: true, in2: false }), [false, false]);
        assert_eq!(machine.process(Testchip2Inputs { in1: true, in2: true }), [true, true]);
    }
}
