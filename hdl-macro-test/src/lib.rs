#[cfg(test)]
mod tests {
    use bumpalo::Bump;
    use hdl::SizedChip;
    use hdl::StructuredData;
    use hdl::{ChipInput, ChipOutput, ChipOutputType, Input, Machine, Nand};
    use hdl_macro::{chip, StructuredData};

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
    fn when_a_output_struct_with_array_and_nonarray_inputs_is_defined_derive_trait_generates_correct_methods(
    ) {
        #[derive(StructuredData, PartialEq, Debug, Clone)]
        struct Test<T> {
            arrayinput1: [T; 2],
            nonarrayinput1: T,
            arrayinput2: [T; 5],
            nonarrayinput2: T,
        }

        let under_test = Test::<bool> {
            nonarrayinput1: true,
            arrayinput1: [false, true],
            nonarrayinput2: false,
            arrayinput2: [false, false, true, false, true],
        };

        let transformed_under_test = Test::<bool>::from_flat(under_test.clone().to_flat());

        assert_eq!(under_test, transformed_under_test);
    }

    #[test]
    fn when_a_chip_is_defined_it_can_be_processed_via_machine() {
        #[chip]
        fn testchip<'a>(
            alloc: &'a Bump,
            in1: &'a ChipInput<'a>,
            in2: &'a ChipInput<'a>,
        ) -> UnaryChipOutput<ChipOutputType<'a>> {
            let nand = Nand::new(&alloc, in1.into(), in2.into());
            UnaryChipOutput {
                out: ChipOutputType::NandOutput(nand),
            }
        }

        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Testchip::from);
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
                Nand::new(alloc, num1[0].into(), num2[0].into()),
                Nand::new(alloc, num1[1].into(), num2[1].into()),
            ];
            TwoBitNumOutput {
                out: bitwise_nand.map(|nand| {
                    ChipOutputType::NandOutput(Nand::new(alloc, bit.into(), nand.into()))
                }),
            }
        }

        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Testchip::from);
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
            let nand = Nand::new(&alloc, in1.into(), in2.into());
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
            let chip = Testchip::new(alloc, in1.into(), in2.into());
            let nand = Nand::new(&alloc, in1.into(), chip.get_out(alloc).out.into());
            BinaryChipOutput::<_> {
                out1: ChipOutputType::NandOutput(nand),
                out2: ChipOutputType::ChipInput(in2),
            }
        }

        let alloc = Bump::new();
        let mut machine = Machine::new(&alloc, Testchip2::from);
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
