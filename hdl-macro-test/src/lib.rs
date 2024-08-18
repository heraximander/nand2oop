#[cfg(test)]
mod tests {
    use bumpalo::Bump;
    use hdl::{ChipInput, ChipOutput, ChipOutputType, Input, Machine, Nand};
    use hdl_macro::chip;
    use hdl::SizedChip;

    #[test]
    fn when_a_chip_is_defined_it_can_be_processed_via_machine() {
        #[chip]
        fn testchip<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 2]) -> [ChipOutputType<'a>;1] {
            let nand = Nand::new(
                &alloc,
                Input::ChipInput(input[0]),
                Input::ChipInput(&input[1]),
            );
            [ChipOutputType::NandOutput(nand)]
        }

        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Testchip::new);
        assert_eq!(machine.process([true,false]), [true]);
        assert_eq!(machine.process([true,true]), [false]);
    }

    #[test]
    fn when_a_chip_is_defined_it_can_be_graphed_via_machine() {
        #[chip]
        fn testchip<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 2]) -> [ChipOutputType<'a>;1] {
            let nand = Nand::new(
                &alloc,
                Input::ChipInput(input[0]),
                Input::ChipInput(input[1]),
            );
            [ChipOutputType::NandOutput(nand)]
        }

        let alloc = Bump::new();
        let machine = Machine::new(&alloc, Testchip::new);
        assert_eq!(machine.graph(), "```mermaid\ngraph TD\nsubgraph Testchip\n1IN(IN)-->0NAND(NAND)\n0NAND(NAND)-->0OUT(OUT)\n0IN(IN)-->0NAND(NAND)\nend\n0INPUT(INPUT)-->0IN(IN)\n0OUT(OUT)-->0OUTPUT(OUTPUT)\n1INPUT(INPUT)-->1IN(IN)\n```");
    }

    #[test]
    fn when_a_nested_chip_is_defined_it_can_be_processed_via_machine() {
        #[chip]
        fn testchip<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 2]) -> [ChipOutputType<'a>;1] {
            let nand = Nand::new(
                &alloc,
                Input::ChipInput(input[0]),
                Input::ChipInput(input[1]),
            );
            [ChipOutputType::NandOutput(nand)]
        }

        #[chip]
        fn testchip2<'a>(alloc: &'a Bump, input: [&'a ChipInput<'a>; 2]) -> [ChipOutputType<'a>;2] {
            let chip = Testchip::new(alloc, [Input::ChipInput(input[0]), Input::ChipInput(input[1])]);
            let nand = Nand::new(
                &alloc,
                Input::ChipInput(input[0]),
                Input::ChipOutput(chip.get_out_sized(alloc)[0]),
            );
            [
                ChipOutputType::NandOutput(nand),
                ChipOutputType::ChipInput(input[1])
            ]
        }

        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Testchip2::new);
        assert_eq!(machine.process([true,false]), [false,false]);
        assert_eq!(machine.process([true,true]), [true,true]);
    }

}
