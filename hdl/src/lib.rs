use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicU32, Ordering},
};

use bumpalo::Bump;

#[cfg(test)]
mod tests {
    use crate::*;

    impl Ord for MermaidLine {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            (self.from.get_label() + &self.to.get_label())
                .cmp(&(other.from.get_label() + &other.to.get_label()))
        }
    }

    impl PartialOrd for MermaidLine {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            (self.from.get_label() + &self.to.get_label())
                .partial_cmp(&(other.from.get_label() + &other.to.get_label()))
        }
    }
    fn sort_mermaid_graph(graph: &mut MermaidGraph) {
        graph.statements.sort();
        for (_, child) in &mut graph.subgraphs {
            sort_mermaid_graph(child);
        }
    }

    #[test]
    fn mermaid_compiles_properly_to_text() {
        struct TestChip {}
        impl<'a> Chip<'a> for TestChip {
            fn get_id(&self) -> String {
                "1".into()
            }

            fn get_label(&self) -> &'static str {
                "TestChip"
            }

            fn get_out_unsized(&'a self, _: &'a Bump) -> &'a [&ChipOutputWrapper] {
                todo!()
            }
        }

        let alloc = Bump::new();
        let in1 = UserInput::new(&alloc);
        let win1 = Input::UserInput(in1);
        let in2 = UserInput::new(&alloc);
        let win2 = Input::UserInput(in2);
        let cin1 = ChipInput::new(&alloc, win1);
        let cin2 = ChipInput::new(&alloc, win2);
        let nand = Nand::new(&alloc, Input::ChipInput(&cin1), Input::ChipInput(&cin2));
        let cout1 = ChipOutput::new(&alloc, ChipOutputType::NandOutput(nand));
        let cout2 = ChipOutput::new(&alloc, ChipOutputType::ChipInput(cin1));
        let outs = [
            Output::new(&ChipOutputWrapper::new(&alloc, &cout1, &TestChip {})),
            Output::new(&ChipOutputWrapper::new(&alloc, &cout2, &TestChip {})),
        ];
        let mermaid_out = graph_outputs(&outs);

        let expected = format!(
            "```mermaid
graph TD
subgraph 1 [TestChip]
{}IN(IN)-->{}OUT(OUT)
{}IN(IN)-->{}NAND(NAND)
{}IN(IN)-->{}NAND(NAND)
{}NAND(NAND)-->{}OUT(OUT)
end
{}INPUT(INPUT)-->{}IN(IN)
{}OUT(OUT)-->{}OUTPUT(OUTPUT)
{}INPUT(INPUT)-->{}IN(IN)
{}OUT(OUT)-->{}OUTPUT(OUTPUT)
```",
            cin1.id,
            cout2.id,
            cin1.id,
            nand.identifier,
            cin2.id,
            nand.identifier,
            nand.identifier,
            cout1.id,
            in1.id,
            cin1.id,
            cout2.id,
            outs[1].identifier,
            in2.id,
            cin2.id,
            cout1.id,
            outs[0].identifier
        );
        let actual = mermaid_out.compile();

        assert_eq!(expected, actual);
    }

    #[test]
    fn multiple_outputs_are_represented_properly_in_mermaid_structure() {
        struct TestChip {}
        impl<'a> Chip<'a> for TestChip {
            fn get_id(&self) -> String {
                "1".into()
            }

            fn get_label(&self) -> &'static str {
                "TestChip"
            }

            fn get_out_unsized(&'a self, _: &'a Bump) -> &'a [&ChipOutputWrapper] {
                todo!()
            }
        }

        let alloc = Bump::new();
        let uin1 = UserInput::new(&alloc);
        let in1 = Input::UserInput(uin1);
        let uin2 = UserInput::new(&alloc);
        let in2 = Input::UserInput(uin2);
        let cin1 = ChipInput::new(&alloc, in1);
        let cin2 = ChipInput::new(&alloc, in2);
        let nand = Nand::new(&alloc, Input::ChipInput(&cin1), Input::ChipInput(&cin2));
        let out1 = ChipOutput::new(&alloc, ChipOutputType::NandOutput(nand));
        let out2 = ChipOutput::new(&alloc, ChipOutputType::ChipInput(cin1));
        let mout1 = Output::new(&ChipOutputWrapper::new(&alloc, &out1, &TestChip {}));
        let mout2 = Output::new(&ChipOutputWrapper::new(&alloc, &out2, &TestChip {}));
        let mouts = [mout1, mout2];
        let mut mermaid_out = graph_outputs(&mouts);

        let mut expected = MermaidGraph {
            statements: Vec::from([
                MermaidLine {
                    from: MermaidNode {
                        identifier: uin1.id,
                        name: "INPUT",
                    },
                    to: MermaidNode {
                        identifier: cin1.id,
                        name: "IN",
                    },
                },
                MermaidLine {
                    from: MermaidNode {
                        identifier: out1.id,
                        name: "OUT",
                    },
                    to: MermaidNode {
                        identifier: mouts[0].identifier,
                        name: "OUTPUT",
                    },
                },
                MermaidLine {
                    from: MermaidNode {
                        identifier: uin2.id,
                        name: "INPUT",
                    },
                    to: MermaidNode {
                        identifier: cin2.id,
                        name: "IN",
                    },
                },
                MermaidLine {
                    from: MermaidNode {
                        identifier: out2.id,
                        name: "OUT",
                    },
                    to: MermaidNode {
                        identifier: mouts[1].identifier,
                        name: "OUTPUT",
                    },
                },
            ]),
            name: "",
            id: "".into(),
            subgraphs: HashMap::from([(
                String::from("1"),
                MermaidGraph {
                    statements: Vec::from([
                        MermaidLine {
                            from: MermaidNode {
                                identifier: cin1.id,
                                name: "IN",
                            },
                            to: MermaidNode {
                                identifier: nand.identifier,
                                name: "NAND",
                            },
                        },
                        MermaidLine {
                            from: MermaidNode {
                                identifier: cin2.id,
                                name: "IN",
                            },
                            to: MermaidNode {
                                identifier: nand.identifier,
                                name: "NAND",
                            },
                        },
                        MermaidLine {
                            from: MermaidNode {
                                identifier: nand.identifier,
                                name: "NAND",
                            },
                            to: MermaidNode {
                                identifier: out1.id,
                                name: "OUT",
                            },
                        },
                        MermaidLine {
                            from: MermaidNode {
                                identifier: cin1.id,
                                name: "IN",
                            },
                            to: MermaidNode {
                                identifier: out2.id,
                                name: "OUT",
                            },
                        },
                    ]),
                    name: "TestChip",
                    subgraphs: HashMap::new(),
                    id: "1".into(),
                },
            )]),
        };
        sort_mermaid_graph(&mut expected);
        sort_mermaid_graph(&mut mermaid_out);

        assert_eq!(expected, mermaid_out);
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
struct MermaidNode {
    identifier: u32,
    name: &'static str,
}

impl MermaidNode {
    fn get_label(&self) -> String {
        format!("{}{}", self.identifier, self.name)
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
struct MermaidLine {
    from: MermaidNode,
    to: MermaidNode,
}

#[derive(Debug, Clone, PartialEq)]
struct MermaidGraph {
    statements: Vec<MermaidLine>,
    name: &'static str,
    id: String,
    subgraphs: HashMap<String, MermaidGraph>,
}

impl MermaidGraph {
    fn new(name: &'static str, id: String) -> MermaidGraph {
        MermaidGraph {
            statements: Vec::new(),
            subgraphs: HashMap::new(),
            id,
            name,
        }
    }

    fn get_subgraph(&mut self, path: &Vec<String>) -> &mut MermaidGraph {
        path.iter()
            .fold(self, |subgraph, id| subgraph.subgraphs.get_mut(id).unwrap())
    }

    fn compile(&self) -> String {
        let mut res = "```mermaid\ngraph TD".to_owned();
        res += &self.compile_subgraph();
        res += "\n```";
        res
    }

    fn compile_subgraph(&self) -> String {
        let mut res = String::new();
        for (_, subgraph) in &self.subgraphs {
            let label = subgraph.name;
            res += &format!("\nsubgraph {} [{}]", subgraph.id, label);
            res += &subgraph.compile_subgraph();
            res += "\nend";
        }
        for statement in &self.statements {
            let left_label = statement.from.get_label();
            let right_label = statement.to.get_label();
            let left_name = statement.from.name;
            let right_name = statement.to.name;
            res += &format!("\n{left_label}({left_name})-->{right_label}({right_name})");
        }
        res
    }
}

// FIXME: work out how to mark struct as non-threadsafe
// maybe it's already ok - it's not Send, Clone or Copy
pub struct Machine<'a, const NINPUT: usize, const NOUT: usize> {
    inputs: [&'a UserInput; NINPUT],
    outputs: [Output<'a>; NOUT],
    iteration: u8,
}

impl<'a, const NINPUT: usize, const NOUT: usize> Machine<'a, NINPUT, NOUT> {
    pub fn new<TChip: SizedChip<'a, NOUT>>(
        alloc: &'a Bump,
        new_fn: fn(&'a Bump, [Input<'a>; NINPUT]) -> &'a TChip,
    ) -> Self {
        let inputs = [0; NINPUT].map(|_| UserInput::new(&alloc));
        let chip = new_fn(&alloc, inputs.map(|in_| Input::UserInput(in_)));
        let outputs = chip.get_out(alloc).map(|out| Output::new(out));
        let machine = Machine {
            inputs,
            outputs,
            iteration: 0,
        };
        machine
    }

    pub fn graph(&self) -> String {
        let graph_map = graph_outputs(&self.outputs);
        graph_map.compile()
    }

    pub fn process(&mut self, input_vals: [bool; NINPUT]) -> [bool; NOUT] {
        for (in_, val) in self.inputs.iter().zip(input_vals) {
            in_.set(val);
        }
        self.iteration += 1;
        let mut res = [true; NOUT];
        for (i, out) in (&self.outputs).iter().enumerate() {
            res[i] = out.output.process(self.iteration);
        }
        res
    }
}

fn graph_outputs<'a>(outputs: &[Output<'a>]) -> MermaidGraph {
    let mut graph_map = MermaidGraph::new("", "".into());
    let mut node_set = HashSet::new();
    for out in outputs.iter().rev() {
        out.graph(&mut graph_map, vec![], &mut node_set);
    }
    graph_map
}

pub struct Output<'a> {
    output: &'a ChipOutputWrapper<'a>,
    identifier: u32,
}

impl<'a> Output<'a> {
    pub fn new(output: &'a ChipOutputWrapper<'a>) -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        Output {
            output,
            identifier: COUNTER.fetch_add(1, Ordering::Relaxed),
        } // FIXME: don't wraparound
    }

    fn graph(
        &self,
        graph_map: &mut MermaidGraph,
        path: Vec<String>,
        node_set: &mut HashSet<String>,
    ) {
        let node = self.output.graph(graph_map, path, node_set);
        graph_map.statements.push(MermaidLine {
            from: node,
            to: MermaidNode {
                identifier: self.identifier,
                name: "OUTPUT",
            },
        });
    }
}

pub struct UserInput {
    value: Cell<bool>,
    id: u32,
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

    fn graph(
        &self,
        _: &mut MermaidGraph,
        _: Vec<String>,
        node_set: &mut HashSet<String>,
    ) -> MermaidNode {
        let node = MermaidNode {
            identifier: self.id,
            name: "INPUT",
        };

        // make sure we haven't already expanded this node
        if node_set.contains(&node.get_label()) {
            return node;
        }
        node_set.insert(node.get_label());
        node
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

    fn graph(
        &self,
        graph_map: &mut MermaidGraph,
        path: Vec<String>,
        node_set: &mut HashSet<String>,
    ) -> MermaidNode {
        match self {
            Input::UserInput(x) => x.graph(graph_map, path, node_set),
            Input::ChipOutput(x) => x.graph(graph_map, path, node_set),
            Input::ChipInput(x) => x.graph(graph_map, path, node_set),
            Input::NandInput(x) => x.graph(graph_map, path, node_set),
        }
    }
}

pub struct ChipInput<'a> {
    in_: Input<'a>,
    id: u32,
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

    fn graph(
        &self,
        graph_map: &mut MermaidGraph,
        path: Vec<String>,
        node_set: &mut HashSet<String>,
    ) -> MermaidNode {
        let node = MermaidNode {
            identifier: self.id,
            name: "IN",
        };

        // make sure we haven't already expanded this node
        if node_set.contains(&node.get_label()) {
            return node;
        }
        node_set.insert(node.get_label());

        let mut new_path = path.clone();
        new_path.pop();
        let prev_node = self.in_.graph(graph_map, new_path.clone(), node_set);

        let current_graph = graph_map.get_subgraph(&new_path);
        current_graph.statements.push(MermaidLine {
            from: prev_node,
            to: node,
        });
        node
    }
}

pub enum ChipOutputType<'a> {
    ChipOutput(&'a ChipOutputWrapper<'a>),
    NandOutput(&'a Nand<'a>),
    ChipInput(&'a ChipInput<'a>),
}

pub struct ChipOutput<'a> {
    out: ChipOutputType<'a>,
    value: Cell<bool>,
    iteration: Cell<u8>,
    id: u32,
}

pub struct ChipOutputWrapper<'a> {
    inner: &'a ChipOutput<'a>,
    parent: &'a dyn Chip<'a>,
}

pub trait Chip<'a> {
    fn get_id(&self) -> String;
    fn get_label(&self) -> &'static str;
    fn get_out_unsized(&'a self, alloc: &'a Bump) -> &'a [&ChipOutputWrapper];
}

pub trait SizedChip<'a, const NOUT: usize>: Chip<'a> {
    fn get_out(&self, alloc: &'a Bump) -> [&'a ChipOutputWrapper; NOUT];
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

    fn graph(
        &self,
        graph_map: &mut MermaidGraph,
        path: Vec<String>,
        node_set: &mut HashSet<String>,
    ) -> MermaidNode {
        let current_graph = graph_map.get_subgraph(&path); // FIXME: this is a bit crap

        // graph the current component
        let node = MermaidNode {
            identifier: self.inner.id,
            name: "OUT",
        };

        // make sure we haven't already expanded this node
        if node_set.contains(&node.get_label()) {
            return node;
        }
        node_set.insert(node.get_label());

        // get a new subgraph because we're at a chip boundary
        let graph_id = self.parent.get_id();
        let new_graph_name = graph_id.clone();
        if !current_graph.subgraphs.contains_key(&new_graph_name) {
            let subgraph = MermaidGraph::new(self.parent.get_label(), graph_id.clone());
            current_graph.subgraphs.insert(graph_id.clone(), subgraph);
        }
        let mut new_path = path.clone();
        new_path.push(new_graph_name);

        // recursively graph the input components
        let prev_node = match self.inner.out {
            ChipOutputType::ChipOutput(out) => out.graph(graph_map, new_path.clone(), node_set),
            ChipOutputType::NandOutput(nand) => nand.graph(graph_map, new_path.clone(), node_set),
            ChipOutputType::ChipInput(in_) => in_.graph(graph_map, new_path.clone(), node_set),
        };

        // add line between this node and the previous
        let subgraph = graph_map.get_subgraph(&new_path);
        subgraph.statements.push(MermaidLine {
            from: prev_node,
            to: node,
        });

        node
    }
}

pub struct Nand<'a> {
    in1: Input<'a>,
    in2: Input<'a>,
    iteration: Cell<u8>,
    value: Cell<bool>,
    identifier: u32,
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

    fn graph(
        &self,
        graph_map: &mut MermaidGraph,
        path: Vec<String>,
        node_set: &mut HashSet<String>,
    ) -> MermaidNode {
        // make sure we haven't already expanded this node
        let node = MermaidNode {
            identifier: self.identifier,
            name: "NAND",
        };
        if node_set.contains(&node.get_label()) {
            return node;
        }
        node_set.insert(node.get_label());

        let from_node_1 = self.in1.graph(graph_map, path.clone(), node_set);
        let from_node_2 = self.in2.graph(graph_map, path.clone(), node_set);

        let current_graph = graph_map.get_subgraph(&path);
        current_graph.statements.push(MermaidLine {
            from: from_node_1,
            to: node,
        });
        current_graph.statements.push(MermaidLine {
            from: from_node_2,
            to: node,
        });

        node
    }
}
