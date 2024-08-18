use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

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

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
enum MermaidStatement {
    Line(MermaidLine),
    Node(MermaidNode),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MermaidGraph {
    statements: Vec<MermaidStatement>,
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
        let mut res = "graph TD".to_owned();
        res += &self.compile_subgraph();
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
            match statement {
                MermaidStatement::Line(line) => {
                    let left_label = line.from.get_label();
                    let right_label = line.to.get_label();
                    let left_name = line.from.name;
                    let right_name = line.to.name;
                    res += &format!("\n{left_label}({left_name})-->{right_label}({right_name})");
                }
                MermaidStatement::Node(node) => {
                    res += &format!("\n{}({})", node.get_label(), node.name);
                }
            }
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
    machine: &Machine<'a, TFam, NINPUT, NOUT>,
    show_chips: HashSet<String>,
) -> MermaidGraph {
    graph_outputs(&machine.outputs, show_chips)
}

fn graph_outputs(outs: &[Output], show_chips: HashSet<String>) -> MermaidGraph {
    let mut graph_map = MermaidGraph::new("", "".into());
    let mut node_set = HashSet::new();
    for out in outs.iter().rev() {
        graph_output(
            out,
            &mut GraphInputs {
                graph_map: &mut graph_map,
                path: vec![],
                node_set: &mut node_set,
                show_chips: &show_chips,
            },
        );
    }
    graph_map
}

fn graph_output(out: &Output<'_>, graph_inputs: &mut GraphInputs<'_>) {
    let node = graph_output_wrapper(out.output, graph_inputs);

    graph_inputs
        .graph_map
        .statements
        .push(MermaidStatement::Line(MermaidLine {
            from: node,
            to: MermaidNode {
                identifier: out.identifier,
                name: "OUTPUT",
            },
        }));
}

struct GraphInputs<'a> {
    graph_map: &'a mut MermaidGraph,
    path: Vec<String>,
    node_set: &'a mut HashSet<String>,
    show_chips: &'a HashSet<String>,
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
            // TODO: find a better way of cloning and updating the inputs struct. Maybe make it copy?
            graph_map: graph_inputs.graph_map,
            path: new_path.clone(),
            node_set: graph_inputs.node_set,
            show_chips: graph_inputs.show_chips,
        },
    );

    if is_node_shown(&graph_inputs.path, graph_inputs.show_chips) {
        let subgraph = graph_inputs.graph_map.get_subgraph(&graph_inputs.path);
        subgraph.statements.push(MermaidStatement::Node(node));

        let current_graph = graph_inputs.graph_map.get_subgraph(&new_path);
        current_graph
            .statements
            .push(MermaidStatement::Line(MermaidLine {
                from: prev_node,
                to: node,
            }));
    }
    node
}

fn graph_output_wrapper(
    out: &ChipOutputWrapper<'_>,
    graph_inputs: &mut GraphInputs<'_>,
) -> MermaidNode {
    let chip_id = out.parent.get_id();
    let mut new_path = graph_inputs.path.clone();
    new_path.push(chip_id.clone());

    // add line between this node and the previous
    let is_node_expanded = is_node_expanded(&new_path, graph_inputs.show_chips);
    let is_node_shown = is_node_shown(&new_path, graph_inputs.show_chips);

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
    if is_node_shown {
        let current_graph = graph_inputs.graph_map.get_subgraph(&graph_inputs.path); // TODO: this is a bit crap
        let new_graph_name = chip_id.clone();
        if !current_graph.subgraphs.contains_key(&new_graph_name) {
            let subgraph = MermaidGraph::new(out.parent.get_label(), chip_id.clone());
            current_graph.subgraphs.insert(chip_id.clone(), subgraph);
        }
    }

    // recursively graph the input components
    let prev_node = match out.inner.get_out() {
        ChipOutputType::ChipOutput(out) => graph_output_wrapper(
            out,
            &mut GraphInputs {
                graph_map: graph_inputs.graph_map,
                path: new_path.clone(),
                node_set: graph_inputs.node_set,
                show_chips: graph_inputs.show_chips,
            },
        ),
        ChipOutputType::NandOutput(nand) => graph_nand(
            nand,
            &mut GraphInputs {
                graph_map: graph_inputs.graph_map,
                path: new_path.clone(),
                node_set: graph_inputs.node_set,
                show_chips: graph_inputs.show_chips,
            },
        ),
        ChipOutputType::ChipInput(in_) => graph_chip_input(
            in_,
            &mut GraphInputs {
                graph_map: graph_inputs.graph_map,
                path: new_path.clone(),
                node_set: graph_inputs.node_set,
                show_chips: graph_inputs.show_chips,
            },
        ),
    };

    if is_node_shown {
        let subgraph = graph_inputs.graph_map.get_subgraph(&new_path);
        if is_node_expanded {
            subgraph
                .statements
                .push(MermaidStatement::Line(MermaidLine {
                    from: prev_node,
                    to: node,
                }));
        } else {
            subgraph.statements.push(MermaidStatement::Node(node))
        }
    }

    node
}

fn is_node_expanded(path: &Vec<String>, show_chips: &HashSet<String>) -> bool {
    path.iter().all(|chip_id| show_chips.contains(chip_id))
}

fn is_node_shown(path: &Vec<String>, show_chips: &HashSet<String>) -> bool {
    path.len() == 0
        || path
            .iter()
            .take(path.len() - 1)
            .all(|chip_id| show_chips.contains(chip_id))
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

    let [in1, in2] = nand.get_inputs();
    let from_node_1 = graph_input(
        in1,
        &mut GraphInputs {
            graph_map: graph_inputs.graph_map,
            path: graph_inputs.path.clone(),
            node_set: graph_inputs.node_set,
            show_chips: graph_inputs.show_chips,
        },
    );
    let from_node_2 = graph_input(
        in2,
        &mut GraphInputs {
            graph_map: graph_inputs.graph_map,
            path: graph_inputs.path.clone(),
            node_set: graph_inputs.node_set,
            show_chips: graph_inputs.show_chips,
        },
    );

    if is_node_expanded(&graph_inputs.path, graph_inputs.show_chips) {
        let current_graph = graph_inputs.graph_map.get_subgraph(&graph_inputs.path);
        current_graph
            .statements
            .push(MermaidStatement::Line(MermaidLine {
                from: from_node_1,
                to: node,
            }));
        current_graph
            .statements
            .push(MermaidStatement::Line(MermaidLine {
                from: from_node_2,
                to: node,
            }));
    }

    node
}

pub fn start_interactive_server<
    'a,
    TFam: StructuredDataFamily<NINPUT, NOUT>,
    const NINPUT: usize,
    const NOUT: usize,
>(
    machine: &Machine<'a, TFam, NINPUT, NOUT>,
    port: u16,
) {
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream, machine);
    }
}

fn handle_connection<
    'a,
    TFam: StructuredDataFamily<NINPUT, NOUT>,
    const NINPUT: usize,
    const NOUT: usize,
>(
    mut stream: TcpStream,
    machine: &Machine<'a, TFam, NINPUT, NOUT>,
) {
    let buf_reader = BufReader::new(&mut stream);
    let lines = buf_reader
        .lines()
        .map(|elem| elem.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    let graph_function = |show_chips| graph_machine(machine, show_chips);
    let response = match get_response(lines, graph_function) {
        Ok(s) => format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            s.len(),
            s
        ),
        Err(_) => "HTTP/1.1 404 NOK\r\n\r\n".into(),
    };
    stream.write_all(response.as_bytes()).unwrap();
}

const HTTP_RESPONSE_TEMPLATE: &str = include_str!("../http/index.html");
fn get_response<'a, F: FnOnce(HashSet<String>) -> MermaidGraph>(
    lines: Vec<String>,
    graph_function: F,
) -> Result<String, ()> {
    let http_line = match lines.iter().find(|line| line.starts_with("GET")) {
        Some(s) => Ok(s),
        None => Err(()),
    }?;

    let is_get = http_line.starts_with("GET");
    if !is_get {
        return Err(());
    }

    let expanded = http_line
        .split_once("?")
        .and_then(|(_, post_params)| post_params.split_once(" "))
        .and_then(|(params, _)| Some(params.split("&")))
        .and_then(|mut params_list| params_list.find(|param| param.starts_with("expanded")))
        .and_then(|expanded_param| Some(expanded_param.replace("expanded=", "")))
        .and_then(|expanded| {
            Some(
                expanded
                    .split(",")
                    .filter(|e| !e.is_empty())
                    .map(String::from)
                    .collect::<Vec<_>>(),
            )
        });
    let show_chips = match expanded {
        Some(e) => HashSet::from_iter(e.into_iter()),
        None => HashSet::new(),
    };

    let graph = graph_function(show_chips);
    let chip_ids = get_subgraph_ids(&graph);

    Ok(HTTP_RESPONSE_TEMPLATE
        .replace("{REPLACE_GRAPH}", &graph.compile())
        .replace(
            "{REPLACE_CHIP_IDS}",
            &chip_ids
                .iter()
                .fold(String::new(), |acc, elem| format!("{}\"{}\",", acc, elem)),
        ))
}

fn get_subgraph_ids<'a>(graph: &'a MermaidGraph) -> HashSet<&'a str> {
    graph
        .subgraphs
        .iter()
        .flat_map(|(k, v)| {
            let mut names = get_subgraph_ids(v);
            names.insert(k);
            names
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{cmp::Ordering, collections::HashMap, vec};

    use bumpalo::Bump;
    use hdl::{Chip, ChipInput, ChipOutput, Input, Output};

    use crate::*;

    impl Ord for MermaidStatement {
        fn cmp(&self, other: &Self) -> Ordering {
            self.partial_cmp(other).unwrap()
        }
    }

    impl PartialOrd for MermaidStatement {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            match self {
                MermaidStatement::Line(self_line) => match other {
                    MermaidStatement::Line(other_line) => (self_line.from.get_label()
                        + &self_line.to.get_label())
                        .partial_cmp(&(other_line.from.get_label() + &other_line.to.get_label())),
                    MermaidStatement::Node(_) => Option::Some(Ordering::Less),
                },
                MermaidStatement::Node(self_node) => match other {
                    MermaidStatement::Line(_) => Option::Some(Ordering::Greater),
                    MermaidStatement::Node(other_node) => {
                        self_node.get_label().partial_cmp(&other_node.get_label())
                    }
                },
            }
        }
    }

    fn sort_mermaid_graph(graph: &mut MermaidGraph) {
        graph.statements.sort();
        for (_, child) in &mut graph.subgraphs {
            sort_mermaid_graph(child);
        }
    }

    #[test]
    fn when_a_request_with_no_query_params_is_passed_in_get_response_returns_success_response_with_internal_implementation_hidden(
    ) {
        let lines = vec!["GET /".into()];
        let resp = get_response(lines, |show_chips| {
            assert_eq!(show_chips, HashSet::new());
            MermaidGraph {
                statements: vec![],
                name: "",
                id: "".into(),
                subgraphs: HashMap::from([(
                    "chip1".into(),
                    MermaidGraph {
                        statements: vec![],
                        name: "",
                        id: "".into(),
                        subgraphs: HashMap::new(),
                    },
                )]),
            }
        })
        .expect("response not valid");
        assert!(
            resp.contains("[\"chip1\",]"),
            "event listener not defined for visible chips"
        );
    }

    #[test]
    fn when_a_request_with_some_query_params_is_passed_in_get_response_returns_success_response_with_internal_implementation_shown(
    ) {
        let lines = vec!["GET /?expanded=chip1, HTTP/1.1".into()];
        get_response(lines, |show_chips| {
            assert_eq!(show_chips, HashSet::from(["chip1".into()]));
            MermaidGraph {
                statements: vec![],
                name: "",
                id: "".into(),
                subgraphs: HashMap::from([(
                    "chip1".into(),
                    MermaidGraph {
                        statements: vec![],
                        name: "",
                        id: "".into(),
                        subgraphs: HashMap::new(),
                    },
                )]),
            }
        })
        .expect("response not valid");
    }

    #[test]
    fn mermaid_compiles_properly_to_text() {
        struct TestChip {}
        const CHIP_ID: &str = "1";
        impl<'a> Chip<'a> for TestChip {
            fn get_id(&self) -> String {
                CHIP_ID.into()
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
        let mermaid_out = graph_outputs(&outs, HashSet::from([CHIP_ID.into()]));

        let expected = format!(
            "graph TD
subgraph 1 [TestChip]
{}IN(IN)
{}IN(IN)-->{}OUT(OUT)
{}IN(IN)
{}IN(IN)-->{}NAND(NAND)
{}IN(IN)-->{}NAND(NAND)
{}NAND(NAND)-->{}OUT(OUT)
end
{}INPUT(INPUT)-->{}IN(IN)
{}OUT(OUT)-->{}OUTPUT(OUTPUT)
{}INPUT(INPUT)-->{}IN(IN)
{}OUT(OUT)-->{}OUTPUT(OUTPUT)",
            cin1.id,
            cin1.id,
            cout2.id,
            cin2.id,
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
        const CHIP_ID: &str = "1";
        impl<'a> Chip<'a> for TestChip {
            fn get_id(&self) -> String {
                CHIP_ID.into()
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
        let mut mermaid_out = graph_outputs(&mouts, HashSet::from([CHIP_ID.into()]));

        let mut expected = MermaidGraph {
            statements: Vec::from([
                MermaidStatement::Line(MermaidLine {
                    from: MermaidNode {
                        identifier: uin1.id,
                        name: "INPUT",
                    },
                    to: MermaidNode {
                        identifier: cin1.id,
                        name: "IN",
                    },
                }),
                MermaidStatement::Line(MermaidLine {
                    from: MermaidNode {
                        identifier: out1.id,
                        name: "OUT",
                    },
                    to: MermaidNode {
                        identifier: mouts[0].identifier,
                        name: "OUTPUT",
                    },
                }),
                MermaidStatement::Line(MermaidLine {
                    from: MermaidNode {
                        identifier: uin2.id,
                        name: "INPUT",
                    },
                    to: MermaidNode {
                        identifier: cin2.id,
                        name: "IN",
                    },
                }),
                MermaidStatement::Line(MermaidLine {
                    from: MermaidNode {
                        identifier: out2.id,
                        name: "OUT",
                    },
                    to: MermaidNode {
                        identifier: mouts[1].identifier,
                        name: "OUTPUT",
                    },
                }),
            ]),
            name: "",
            id: "".into(),
            subgraphs: HashMap::from([(
                String::from("1"),
                MermaidGraph {
                    statements: Vec::from([
                        MermaidStatement::Node(MermaidNode {
                            identifier: cin1.id,
                            name: "IN",
                        }),
                        MermaidStatement::Node(MermaidNode {
                            identifier: cin2.id,
                            name: "IN",
                        }),
                        MermaidStatement::Line(MermaidLine {
                            from: MermaidNode {
                                identifier: cin1.id,
                                name: "IN",
                            },
                            to: MermaidNode {
                                identifier: nand.identifier,
                                name: "NAND",
                            },
                        }),
                        MermaidStatement::Line(MermaidLine {
                            from: MermaidNode {
                                identifier: cin2.id,
                                name: "IN",
                            },
                            to: MermaidNode {
                                identifier: nand.identifier,
                                name: "NAND",
                            },
                        }),
                        MermaidStatement::Line(MermaidLine {
                            from: MermaidNode {
                                identifier: nand.identifier,
                                name: "NAND",
                            },
                            to: MermaidNode {
                                identifier: out1.id,
                                name: "OUT",
                            },
                        }),
                        MermaidStatement::Line(MermaidLine {
                            from: MermaidNode {
                                identifier: cin1.id,
                                name: "IN",
                            },
                            to: MermaidNode {
                                identifier: out2.id,
                                name: "OUT",
                            },
                        }),
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

    #[test]
    fn when_a_chip_is_not_in_the_show_nodes_set_but_its_parent_is_only_the_inputs_and_outputs_are_rendered(
    ) {
        struct TestChip1 {}
        const CHIP_ID_1: &str = "1";
        impl<'a> Chip<'a> for TestChip1 {
            fn get_id(&self) -> String {
                CHIP_ID_1.into()
            }

            fn get_label(&self) -> &'static str {
                "TestChip1"
            }
        }

        struct TestChip2 {}
        const CHIP_ID_2: &str = "2";
        impl<'a> Chip<'a> for TestChip2 {
            fn get_id(&self) -> String {
                CHIP_ID_2.into()
            }

            fn get_label(&self) -> &'static str {
                "TestChip2"
            }
        }

        let alloc = Bump::new();
        let uin1 = UserInput::new(&alloc);
        let in1 = Input::UserInput(uin1);
        let uin2 = UserInput::new(&alloc);
        let in2 = Input::UserInput(uin2);
        let c1in1 = ChipInput::new(&alloc, in1);
        let c1in2 = ChipInput::new(&alloc, in2);
        let c2in1 = ChipInput::new(&alloc, Input::ChipInput(c1in1));
        let c2in2 = ChipInput::new(&alloc, Input::ChipInput(c1in2));
        let nand = Nand::new(&alloc, Input::ChipInput(&c2in1), Input::ChipInput(&c2in2));
        let c2out = ChipOutput::new(&alloc, ChipOutputType::NandOutput(nand));
        let c1out = ChipOutput::new(
            &alloc,
            ChipOutputType::ChipOutput(ChipOutputWrapper::new(&alloc, c2out, &TestChip2 {})),
        );
        let mout1 = Output::new(&ChipOutputWrapper::new(&alloc, &c1out, &TestChip1 {}));
        let mouts = [mout1];
        let mermaid_out = graph_outputs(&mouts, HashSet::from([]));

        assert!(
            mermaid_out.subgraphs.contains_key(CHIP_ID_1),
            "_TestChip1_ should be shown"
        );
        let testchip1_has_only_input_and_output_nodes = mermaid_out.subgraphs[CHIP_ID_1]
            .statements
            .iter()
            .all(|s| match s {
                MermaidStatement::Node(x) => x.name == "IN" || x.name == "OUT",
                MermaidStatement::Line(_) => true,
            });
        assert!(
            testchip1_has_only_input_and_output_nodes,
            "_TestChip1_ should only display input and output nodes"
        );
        assert!(
            !mermaid_out.subgraphs[CHIP_ID_1]
                .subgraphs
                .contains_key(CHIP_ID_2),
            "_TestChip2_ should be hidden"
        );
        assert_eq!(mermaid_out.subgraphs[CHIP_ID_1].subgraphs.len(), 0);
    }
}
