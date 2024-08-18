use std::{
    cell::Cell,
    marker::PhantomData,
    sync::atomic::{AtomicU32, Ordering},
};

use bumpalo::Bump;

// FIXME: work out how to mark struct as non-threadsafe
// maybe it's already ok - it's not Send, Clone or Copy
pub struct Machine<
    'a,
    TFam: StructuredDataFamily<NINPUT, NOUT>,
    const NINPUT: usize,
    const NOUT: usize,
> {
    inputs: [&'a UserInput; NINPUT],
    pub outputs: [Output<'a>; NOUT],
    iteration: u8,
    phantom_data: PhantomData<TFam>,
}

pub trait StructuredData<T, const NINPUT: usize> {
    fn from_flat(input: [T; NINPUT]) -> Self;
    fn to_flat(self) -> [T; NINPUT];
}

pub trait StructuredDataFamily<const NINPUT: usize, const NOUT: usize> {
    type StructuredInput<T>: StructuredData<T, NINPUT>;
    type StructuredOutput<T>: StructuredData<T, NOUT>;
}

impl<'a, TFam: StructuredDataFamily<NINPUT, NOUT>, const NINPUT: usize, const NOUT: usize>
    Machine<'a, TFam, NINPUT, NOUT>
{
    pub fn new<TChip: SizedChip<'a, TFam, NOUT, NINPUT>>(
        alloc: &'a Bump,
        new_fn: fn(&'a Bump, TFam::StructuredInput<Input<'a>>) -> &'a TChip,
    ) -> Self {
        let inputs = [0; NINPUT].map(|_| UserInput::new(&alloc));
        let input_struct =
            TFam::StructuredInput::from_flat(inputs.map(|in_| Input::UserInput(in_)));
        let chip = new_fn(&alloc, input_struct);
        let outputs = chip.get_out(alloc).to_flat().map(|out| Output::new(out));
        let machine = Machine {
            inputs,
            outputs,
            iteration: 0,
            phantom_data: PhantomData,
        };
        machine
    }

    pub fn process(&mut self, input: TFam::StructuredInput<bool>) -> TFam::StructuredOutput<bool> {
        let flat_input = input.to_flat();
        for (in_, val) in self.inputs.iter().zip(flat_input) {
            in_.set(val);
        }
        self.iteration += 1;
        let mut res = [true; NOUT];
        for (i, out) in (&self.outputs).iter().enumerate() {
            res[i] = out.output.process(self.iteration);
        }
        TFam::StructuredOutput::from_flat(res)
    }
}

pub struct Output<'a> {
    pub output: &'a ChipOutputWrapper<'a>,
    pub identifier: u32,
}

impl<'a> Output<'a> {
    pub fn new(output: &'a ChipOutputWrapper<'a>) -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        Output {
            output,
            identifier: COUNTER.fetch_add(1, Ordering::Relaxed),
        } // FIXME: don't wraparound
    }
}

pub struct UserInput {
    value: Cell<bool>,
    pub id: u32,
}

impl UserInput {
    pub fn new(alloc: &Bump) -> &Self {
        Self::from(alloc, false)
    }

    pub fn from(alloc: &Bump, val: bool) -> &Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        alloc.alloc(UserInput {
            value: Cell::new(val),
            id: COUNTER.fetch_add(1, Ordering::Relaxed),
        })
    }

    pub fn set(&self, value: bool) {
        self.value.set(value);
    }
}

#[derive(Copy, Clone)]
pub enum Input<'a> {
    UserInput(&'a UserInput),
    ChipOutput(&'a ChipOutputWrapper<'a>),
    ChipInput(&'a ChipInput<'a>),
    NandInput(&'a Nand<'a>),
}

impl Input<'_> {
    fn process(&self, iteration: u8) -> bool {
        match self {
            Input::UserInput(in_) => in_.value.get(),
            Input::ChipOutput(out) => out.inner.process(iteration),
            Input::ChipInput(in_) => in_.process(iteration),
            Input::NandInput(nand) => nand.process(iteration),
        }
    }
}

pub struct ChipInput<'a> {
    pub in_: Input<'a>,
    pub id: u32,
}

impl<'a> ChipInput<'a> {
    pub fn new(alloc: &'a Bump, in_: Input<'a>) -> &'a Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        alloc.alloc(ChipInput {
            in_,
            id: COUNTER.fetch_add(1, Ordering::Relaxed),
        })
    }

    fn process(&self, iteration: u8) -> bool {
        self.in_.process(iteration)
    }
}

pub enum ChipOutputType<'a> {
    ChipOutput(&'a ChipOutputWrapper<'a>),
    NandOutput(&'a Nand<'a>),
    ChipInput(&'a ChipInput<'a>),
}

pub struct ChipOutput<'a> {
    pub out: ChipOutputType<'a>,
    value: Cell<bool>,
    iteration: Cell<u8>,
    pub id: u32,
}

pub struct ChipOutputWrapper<'a> {
    pub inner: &'a ChipOutput<'a>,
    pub parent: &'a dyn Chip<'a>,
}

pub trait Chip<'a> {
    fn get_id(&self) -> String;
    fn get_label(&self) -> &'static str;
}

// SizedChip requires knowledge of input and output sizes. If we folded this trait in
// to Chip, then each node of the chip graph would need to know the sizes of the chips
// feeding in to it. This gave me a lot of problems, and as we only need `::get_out()`
// when constructing chips when we know the concrete type of the chip anyway, I decided
// to split the functionality in to two traits
pub trait SizedChip<
    'a,
    TDataFam: StructuredDataFamily<NINPUT, NOUT>,
    const NOUT: usize,
    const NINPUT: usize,
>: Chip<'a>
{
    fn get_out(&self, alloc: &'a Bump) -> TDataFam::StructuredOutput<&'a ChipOutputWrapper>;
}

impl<'a> ChipOutput<'a> {
    pub fn new(alloc: &'a Bump, out: ChipOutputType<'a>) -> &'a Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        alloc.alloc(ChipOutput {
            out,
            iteration: Cell::new(0),
            value: Cell::new(false),
            id: COUNTER.fetch_add(1, Ordering::Relaxed),
        })
    }

    fn process(&self, iteration: u8) -> bool {
        if self.iteration.get() == iteration {
            return self.value.get();
        };
        let res = match self.out {
            ChipOutputType::ChipOutput(out) => out.inner.process(iteration),
            ChipOutputType::NandOutput(nand) => nand.process(iteration),
            ChipOutputType::ChipInput(in_) => in_.process(iteration),
        };
        self.iteration.set(iteration);
        self.value.set(res);
        res
    }
}

impl<'a> ChipOutputWrapper<'a> {
    pub fn new(alloc: &'a Bump, inner: &'a ChipOutput<'a>, parent: &'a impl Chip<'a>) -> &'a Self {
        alloc.alloc(ChipOutputWrapper { inner, parent })
    }

    fn process(&self, iteration: u8) -> bool {
        self.inner.process(iteration)
    }
}

pub struct Nand<'a> {
    pub in1: Input<'a>,
    pub in2: Input<'a>,
    iteration: Cell<u8>,
    value: Cell<bool>,
    pub identifier: u32,
}

impl<'a> Nand<'a> {
    pub fn new(alloc: &'a Bump, in1: Input<'a>, in2: Input<'a>) -> &'a Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        alloc.alloc(Nand {
            in1,
            in2,
            iteration: Cell::new(0),
            value: Cell::new(false),
            identifier: COUNTER.fetch_add(1, Ordering::Relaxed),
        })
    }

    fn process(&self, iteration: u8) -> bool {
        if iteration == self.iteration.get() {
            return self.value.get();
        }
        let in1 = self.in1.process(iteration);
        let in2 = self.in2.process(iteration);
        let res = !(in1 && in2);
        self.iteration.set(iteration);
        self.value.set(res);
        res
    }
}
