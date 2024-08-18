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

impl<'a> Into<Input<'a>> for &'a UserInput {
    fn into(self) -> Input<'a> {
        Input::UserInput(self)
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
    pub label: String, // note that this could instead be a &'static str
                       // it would make the macros slightly more complex
}

impl<'a> ChipInput<'a> {
    pub fn new(alloc: &'a Bump, in_: Input<'a>, label: String) -> &'a Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        alloc.alloc(ChipInput {
            in_,
            id: COUNTER.fetch_add(1, Ordering::Relaxed),
            label,
        })
    }

    fn process(&self, iteration: u8) -> bool {
        self.in_.process(iteration)
    }
}

impl<'a> Into<Input<'a>> for &'a ChipInput<'a> {
    fn into(self) -> Input<'a> {
        Input::ChipInput(self)
    }
}

#[derive(Copy, Clone)]
pub enum ChipOutputType<'a> {
    ChipOutput(&'a ChipOutputWrapper<'a>),
    NandOutput(&'a Nand<'a>),
    ChipInput(&'a ChipInput<'a>),
}

pub struct ChipOutput<'a> {
    out: Cell<Option<ChipOutputType<'a>>>,
    value: Cell<bool>,
    iteration: Cell<u8>,
    pub id: u32,
}

pub struct ChipOutputWrapper<'a> {
    pub inner: &'a ChipOutput<'a>,
    pub parent: &'a dyn Chip<'a>,
}

impl<'a> Into<Input<'a>> for &'a ChipOutputWrapper<'a> {
    fn into(self) -> Input<'a> {
        Input::ChipOutput(self)
    }
}

impl<'a> Into<ChipOutputType<'a>> for &'a ChipOutputWrapper<'a> {
    fn into(self) -> ChipOutputType<'a> {
        ChipOutputType::ChipOutput(self)
    }
}

pub trait Chip<'a> {
    fn get_id(&self) -> String;
    fn get_label(&self) -> &'static str;
}

pub trait DefaultChip<
    'a,
    TDataFam: StructuredDataFamily<NINPUT, NOUT>,
    const NINPUT: usize,
    const NOUT: usize,
>: Chip<'a>
{
    fn new(alloc: &'a Bump) -> &mut Self;
    fn set_inputs(&'a self, alloc: &'a Bump, input: TDataFam::StructuredInput<Input<'a>>);
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
        ChipOutput::<'a>::new_from_option(alloc, Some(out))
    }

    pub fn new_from_option(alloc: &'a Bump, out: Option<ChipOutputType<'a>>) -> &'a Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        alloc.alloc(ChipOutput {
            out: Cell::new(out),
            iteration: Cell::new(0),
            value: Cell::new(false),
            id: COUNTER.fetch_add(1, Ordering::Relaxed),
        })
    }

    pub fn set_out(&self, out: ChipOutputType<'a>) {
        self.out.set(Some(out));
    }

    pub fn get_out(&self) -> ChipOutputType<'a> {
        // we're fine to unwrap the below as we assume that all references
        // are Some by the time the graph is processed. If not, that's because
        // a user has been using APIs they shouldn't have (see create_subchip())
        self.out.get().unwrap()
    }

    fn process(&self, iteration: u8) -> bool {
        if self.iteration.get() == iteration {
            return self.value.get();
        };

        let res = match self.get_out() {
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
    in1: Cell<Option<Input<'a>>>,
    in2: Cell<Option<Input<'a>>>,
    iteration: Cell<u8>,
    value: Cell<bool>,
    pub identifier: u32,
}

pub struct NandInputs<T> {
    pub in1: T,
    pub in2: T,
}

impl<T> StructuredData<T, 2> for NandInputs<T> {
    fn from_flat(input: [T; 2]) -> Self {
        let [in1, in2] = input;
        NandInputs { in1, in2 }
    }

    fn to_flat(self) -> [T; 2] {
        [self.in1, self.in2]
    }
}

pub struct NandOutputs<T> {
    out: T,
}

impl<T> StructuredData<T, 1> for NandOutputs<T> {
    fn from_flat(input: [T; 1]) -> Self {
        let [out] = input;
        NandOutputs { out }
    }

    fn to_flat(self) -> [T; 1] {
        [self.out]
    }
}

impl<'a> Nand<'a> {
    pub fn new(alloc: &'a Bump, in1: Input<'a>, in2: Input<'a>) -> &'a Self {
        let nand: &mut Nand<'a> = DefaultChip::new(alloc);
        nand.in1.set(Some(in1));
        nand.in2.set(Some(in2));
        nand
    }

    pub fn get_inputs(&self) -> [Input<'a>; 2] {
        // note that we could get rid of these unwraps()
        // an idea is to use a different struct, PartialNand, while building
        // the partial chips, and then returning Nand only when the inputs
        // are provided. This would however invalidate the previous memory
        // references, so I've put this in the too hard basket for now and
        // just trust this library to keep Nand gates with Some() inputs
        [self.in1.get().unwrap(), self.in2.get().unwrap()]
    }

    fn process(&self, iteration: u8) -> bool {
        let in1 = match self.in1.get() {
            Some(x) => x,
            // should never get here
            None => panic!("NAND must have two inputs before processing"),
        };
        let in2 = match self.in2.get() {
            Some(x) => x,
            // should never get here
            None => panic!("NAND must have two inputs before processing"),
        };
        if iteration == self.iteration.get() {
            return self.value.get();
        }
        // we set the iteration first in case there's a circular reference
        // then the reference returns the previous iteration value
        // note that if this evaluator is modified to work concurrently
        // this may be unsafe
        self.iteration.set(iteration);
        let in1 = in1.process(iteration);
        let in2 = in2.process(iteration);
        let res = !(in1 && in2);
        self.value.set(res);
        res
    }
}

pub struct NandInputsFamily;

impl StructuredDataFamily<2, 1> for NandInputsFamily {
    type StructuredInput<T> = NandInputs<T>;
    type StructuredOutput<T> = NandOutputs<T>;
}

impl<'a> Into<Input<'a>> for &'a Nand<'a> {
    fn into(self) -> Input<'a> {
        Input::NandInput(self)
    }
}

impl<'a> Chip<'a> for Nand<'a> {
    fn get_id(&self) -> String {
        self.identifier.to_string()
    }

    fn get_label(&self) -> &'static str {
        "NAND"
    }
}

impl<'a> DefaultChip<'a, NandInputsFamily, 2, 1> for Nand<'a> {
    fn new(alloc: &Bump) -> &mut Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        alloc.alloc(Nand {
            in1: Cell::new(None),
            in2: Cell::new(None),
            iteration: Cell::new(0),
            value: Cell::new(false),
            identifier: COUNTER.fetch_add(1, Ordering::Relaxed),
        })
    }

    fn set_inputs(
        &self,
        _: &Bump,
        input: <NandInputsFamily as StructuredDataFamily<2, 1>>::StructuredInput<Input<'a>>,
    ) {
        self.in1.set(Some(input.in1));
        self.in2.set(Some(input.in2));
    }
}

impl<'a> Into<ChipOutputType<'a>> for &'a Nand<'a> {
    fn into(self) -> ChipOutputType<'a> {
        ChipOutputType::NandOutput(self)
    }
}

pub trait ArrayInto<T> {
    fn ainto(self) -> T;
}

impl<'a, TIn: Into<TOut>, TOut, const N: usize> ArrayInto<[TOut; N]> for [TIn; N] {
    fn ainto(self) -> [TOut; N] {
        self.map(|e| e.into())
    }
}

pub fn create_subchip<
    'a,
    const NINPUT1: usize,
    const NOUT1: usize,
    const NINPUT2: usize,
    const NOUT2: usize,
    TDataFam1: StructuredDataFamily<NINPUT1, NOUT1>,
    TDataFam2: StructuredDataFamily<NINPUT2, NOUT2>,
    T1: DefaultChip<'a, TDataFam1, NINPUT1, NOUT1>,
    T2: DefaultChip<'a, TDataFam2, NINPUT2, NOUT2>,
>(
    alloc: &'a Bump,
    in1: &dyn Fn(
        (&'a T2,),
    )
        -> <TDataFam1 as StructuredDataFamily<NINPUT1, NOUT1>>::StructuredInput<Input<'a>>,
    in2: &dyn Fn(
        (&'a T1,),
    )
        -> <TDataFam2 as StructuredDataFamily<NINPUT2, NOUT2>>::StructuredInput<Input<'a>>, // note: I would rather the two closures _not_ involve dynamic dispatch, but then
                                                                                            // I don't get type inference on its parameters
) -> (&'a T1, &'a T2) {
    let chip1 = T1::new(alloc);
    let chip2 = T2::new(alloc);

    chip1.set_inputs(alloc, in1((chip2,)));
    chip2.set_inputs(alloc, in2((chip1,)));

    (chip1, chip2)
}
