use std::collections::{HashMap, HashSet};

use hdl::{
    ChipInput, ChipOutputType, ChipOutputWrapper, Input, Machine, Nand, Output,
    StructuredDataFamily, UserInput,
};

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
pub struct MermaidGraph {
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

    pub fn compile(&self) -> String {
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

pub fn graph_machine<
    'a,
    TFam: StructuredDataFamily<NINPUT, NOUT>,
    const NINPUT: usize,
    const NOUT: usize,
>(
    machine: Machine<'a, TFam, NINPUT, NOUT>,
) -> MermaidGraph {
    graph_outputs(&machine.outputs)
}

fn graph_outputs(outs: &[Output]) -> MermaidGraph {
    let mut graph_map = MermaidGraph::new("", "".into());
    let mut node_set = HashSet::new();
    for out in outs.iter().rev() {
        graph_output(
            out,
            &mut GraphInputs {
                graph_map: &mut graph_map,
                path: vec![],
                node_set: &mut node_set,
            },
        );
    }
    graph_map
}

fn graph_output(out: &Output<'_>, graph_inputs: &mut GraphInputs<'_>) {
    let node = graph_output_wrapper(out.output, graph_inputs);
    graph_inputs.graph_map.statements.push(MermaidLine {
        from: node,
        to: MermaidNode {
            identifier: out.identifier,
            name: "OUTPUT",
        },
    });
}

struct GraphInputs<'a> {
    graph_map: &'a mut MermaidGraph,
    path: Vec<String>,
    node_set: &'a mut HashSet<String>,
}

fn graph_user_input(in_: &UserInput, node_set: &mut HashSet<String>) -> MermaidNode {
    let node = MermaidNode {
        identifier: in_.id,
        name: "INPUT",
    };

    // make sure we haven't already expanded this node
    if node_set.contains(&node.get_label()) {
        return node;
    }
    node_set.insert(node.get_label());
    node
}

fn graph_input(in_: Input<'_>, graph_inputs: &mut GraphInputs<'_>) -> MermaidNode {
    match in_ {
        Input::UserInput(x) => graph_user_input(x, graph_inputs.node_set),
        Input::ChipOutput(x) => graph_output_wrapper(x, graph_inputs),
        Input::ChipInput(x) => graph_chip_input(x, graph_inputs),
        Input::NandInput(x) => graph_nand(x, graph_inputs),
    }
}

fn graph_chip_input(in_: &ChipInput<'_>, graph_inputs: &mut GraphInputs<'_>) -> MermaidNode {
    let node = MermaidNode {
        identifier: in_.id,
        name: "IN",
    };

    // make sure we haven't already expanded this node
    if graph_inputs.node_set.contains(&node.get_label()) {
        return node;
    }
    graph_inputs.node_set.insert(node.get_label());

    let mut new_path = graph_inputs.path.clone();
    new_path.pop();
    let prev_node = graph_input(
        in_.in_,
        &mut GraphInputs {
            graph_map: graph_inputs.graph_map,
            path: new_path.clone(),
            node_set: graph_inputs.node_set,
        },
    );

    let current_graph = graph_inputs.graph_map.get_subgraph(&new_path);
    current_graph.statements.push(MermaidLine {
        from: prev_node,
        to: node,
    });
    node
}

fn graph_output_wrapper(
    out: &ChipOutputWrapper<'_>,
    graph_inputs: &mut GraphInputs<'_>,
) -> MermaidNode {
    let current_graph = graph_inputs.graph_map.get_subgraph(&graph_inputs.path); // TODO: this is a bit crap

    // graph the current component
    let node = MermaidNode {
        identifier: out.inner.id,
        name: "OUT",
    };

    // make sure we haven't already expanded this node
    if graph_inputs.node_set.contains(&node.get_label()) {
        return node;
    }
    graph_inputs.node_set.insert(node.get_label());

    // get a new subgraph because we're at a chip boundary
    let graph_id = out.parent.get_id();
    let new_graph_name = graph_id.clone();
    if !current_graph.subgraphs.contains_key(&new_graph_name) {
        let subgraph = MermaidGraph::new(out.parent.get_label(), graph_id.clone());
        current_graph.subgraphs.insert(graph_id.clone(), subgraph);
    }
    let mut new_path = graph_inputs.path.clone();
    new_path.push(new_graph_name);

    // recursively graph the input components
    let prev_node = match out.inner.out {
        ChipOutputType::ChipOutput(out) => graph_output_wrapper(
            out,
            &mut GraphInputs {
                graph_map: graph_inputs.graph_map,
                path: new_path.clone(),
                node_set: graph_inputs.node_set,
            },
        ),
        ChipOutputType::NandOutput(nand) => graph_nand(
            nand,
            &mut GraphInputs {
                graph_map: graph_inputs.graph_map,
                path: new_path.clone(),
                node_set: graph_inputs.node_set,
            },
        ),
        ChipOutputType::ChipInput(in_) => graph_chip_input(
            in_,
            &mut GraphInputs {
                graph_map: graph_inputs.graph_map,
                path: new_path.clone(),
                node_set: graph_inputs.node_set,
            },
        ),
    };

    // add line between this node and the previous
    let subgraph = graph_inputs.graph_map.get_subgraph(&new_path);
    subgraph.statements.push(MermaidLine {
        from: prev_node,
        to: node,
    });

    node
}

fn graph_nand(nand: &Nand<'_>, graph_inputs: &mut GraphInputs<'_>) -> MermaidNode {
    // make sure we haven't already expanded this node
    let node = MermaidNode {
        identifier: nand.identifier,
        name: "NAND",
    };
    if graph_inputs.node_set.contains(&node.get_label()) {
        return node;
    }
    graph_inputs.node_set.insert(node.get_label());

    let from_node_1 = graph_input(
        nand.in1,
        &mut GraphInputs {
            graph_map: graph_inputs.graph_map,
            path: graph_inputs.path.clone(),
            node_set: graph_inputs.node_set,
        },
    );
    let from_node_2 = graph_input(
        nand.in2,
        &mut GraphInputs {
            graph_map: graph_inputs.graph_map,
            path: graph_inputs.path.clone(),
            node_set: graph_inputs.node_set,
        },
    );

    let current_graph = graph_inputs.graph_map.get_subgraph(&graph_inputs.path);
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use bumpalo::Bump;
    use hdl::{Chip, ChipInput, ChipOutput, Input, Output};

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
