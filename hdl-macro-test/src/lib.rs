#[cfg(test)]
mod tests {
    use bumpalo::Bump;
    use hdl::SizedChip;
    use hdl::{ChipInput, ChipOutput, ChipOutputType, Input, Machine, Nand};
    use hdl_macro::{chip, StructuredData};

    /*
    TODO:
    1. Add ADR
    1. Make sure this new construct works for all my current use cases
    2. Add tests for the StructuredData derive macro
    3. Clean and and robustify the macro
    4. Make sure everything I changed in hdl/lib.rs is all good
    5. Document everything I haven't done in this commit as tech debt or potential features
    6. commit
    7. Add issue tracking + issue tracking ADR
    8. Add visualisation project - probs first abstract visualisation out of core hdl/ pkg
     */
    #[derive(StructuredData, PartialEq, Debug)]
    struct TwoBitNumOutput<T> {
        out: [T; 2],
    }

    #[derive(StructuredData, PartialEq, Debug)]
    struct UnaryChipOutput<T> {
        out: T,
    }

    #[derive(StructuredData, PartialEq, Debug)]
    struct BinaryChipOutput<T> {
        out1: T,
        out2: T,
    }

    #[test]
    fn when_a_chip_is_defined_it_can_be_processed_via_machine() {
        #[derive(StructuredData)]
        struct Test<T> {
            out: [T; 1], // FIXME: delete this struct
            ou1: T,
        }

        #[chip]
        fn testchip<'a>(
            alloc: &'a Bump,
            in1: &'a ChipInput<'a>,
            in2: &'a ChipInput<'a>,
        ) -> UnaryChipOutput<ChipOutputType<'a>> {
            let nand = Nand::new(&alloc, Input::ChipInput(in1), Input::ChipInput(in2));
            UnaryChipOutput {
                out: ChipOutputType::NandOutput(nand),
            }
        }

        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Testchip::new);
        assert_eq!(
            machine.process(TestchipInputs {
                in1: true,
                in2: false
            }),
            UnaryChipOutput { out: true }
        );
        assert_eq!(
            machine.process(TestchipInputs {
                in1: true,
                in2: true
            }),
            UnaryChipOutput { out: false }
        );
    }

    #[test]
    fn when_a_chip_is_defined_with_vector_inputs_it_can_be_processed_via_machine() {
        #[chip]
        fn testchip<'a>(
            alloc: &'a Bump,
            num1: [&'a ChipInput<'a>; 2],
            num2: [&'a ChipInput<'a>; 2],
            bit: &'a ChipInput<'a>,
        ) -> TwoBitNumOutput<ChipOutputType<'a>> {
            let bitwise_nand = [
                Nand::new(alloc, Input::ChipInput(num1[0]), Input::ChipInput(num2[0])),
                Nand::new(alloc, Input::ChipInput(num1[1]), Input::ChipInput(num2[1])),
            ];
            TwoBitNumOutput {
                out: bitwise_nand.map(|nand| {
                    ChipOutputType::NandOutput(Nand::new(
                        alloc,
                        Input::ChipInput(bit),
                        Input::NandInput(nand),
                    ))
                }),
            }
        }

        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Testchip::new);
        assert_eq!(
            machine.process(TestchipInputs {
                num1: [true, true],
                num2: [true, false],
                bit: true
            }),
            TwoBitNumOutput { out: [true, false] }
        );
        assert_eq!(
            machine.process(TestchipInputs {
                num1: [true, true],
                num2: [true, false],
                bit: false
            }),
            TwoBitNumOutput { out: [true, true] }
        );
    }

    #[test]
    fn when_a_nested_chip_is_defined_it_can_be_processed_via_machine() {
        #[chip]
        fn testchip<'a>(
            alloc: &'a Bump,
            in1: &'a ChipInput<'a>,
            in2: &'a ChipInput<'a>,
        ) -> UnaryChipOutput<ChipOutputType<'a>> {
            let nand = Nand::new(&alloc, Input::ChipInput(in1), Input::ChipInput(in2));
            UnaryChipOutput {
                out: ChipOutputType::NandOutput(nand),
            }
        }

        #[chip]
        fn testchip2<'a>(
            alloc: &'a Bump,
            in1: &'a ChipInput<'a>,
            in2: &'a ChipInput<'a>,
        ) -> BinaryChipOutput<ChipOutputType<'a>> {
            let chip = Testchip::new(
                alloc,
                TestchipInputs {
                    in1: Input::ChipInput(in1),
                    in2: Input::ChipInput(in2),
                },
            );
            let nand = Nand::new(
                &alloc,
                Input::ChipInput(in1),
                Input::ChipOutput(chip.get_out(alloc).out),
            );
            BinaryChipOutput::<_> {
                out1: ChipOutputType::NandOutput(nand),
                out2: ChipOutputType::ChipInput(in2),
            }
        }

        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Testchip2::new);
        assert_eq!(
            machine.process(Testchip2Inputs {
                in1: true,
                in2: false
            }),
            BinaryChipOutput {
                out1: false,
                out2: false
            }
        );
        assert_eq!(
            machine.process(Testchip2Inputs {
                in1: true,
                in2: true
            }),
            BinaryChipOutput {
                out1: true,
                out2: true
            }
        );
    }
}
