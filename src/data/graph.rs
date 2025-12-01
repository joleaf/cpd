use crate::data::edge::Edge;
use crate::data::vertex::Vertex;
use std::fs::File;
use std::io::BufRead;
use std::path::Path;
use std::{fmt, io};

#[derive(Debug)]
pub struct GraphSetParseError {
    message: String,
}

impl fmt::Display for GraphSetParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug)]
pub struct Graph {
    pub id: usize,
    pub vertices: Vec<Vertex>,
}

impl Graph {
    pub fn new(id: usize) -> Graph {
        Graph {
            id,
            vertices: Vec::with_capacity(32),
        }
    }

    pub fn create_vertex(&mut self) -> &mut Vertex {
        let vertex = Vertex::new(self.vertices.len(), None, None);
        self.vertices.push(vertex);
        self.get_last_vertex()
    }

    pub fn create_vertex_with_data(&mut self, label: usize, vertex_type: usize) -> &mut Vertex {
        let vertex = self.create_vertex();
        vertex.label = label;
        vertex.vertex_type = vertex_type;
        vertex
    }

    fn get_last_vertex(&mut self) -> &mut Vertex {
        self.vertices.last_mut().unwrap()
    }

    pub fn has_vertex_with_id(&self, id: &usize) -> bool {
        self.vertices.len() > *id
    }

    pub fn get_vertices_by_type(&self, vertex_type: usize) -> Vec<&Vertex> {
        self.vertices
            .iter()
            .filter(|vertex| vertex.vertex_type == vertex_type)
            .collect()
    }

    pub fn graphs_set_from_file<P>(path: P) -> Result<Vec<Graph>, GraphSetParseError>
    where
        P: AsRef<Path>,
    {
        let mut graph_list = Vec::new();
        let mut graph_id = 0;
        let mut current_graph: Graph = Graph::new(usize::MAX);
        let line_reader = read_lines(path);
        match line_reader {
            Ok(lines) => {
                for data_line in lines.map_while(Result::ok) {
                    let mut data = data_line.split(" ");
                    if let Some(data_type) = data.next() {
                        match data_type {
                            "t" => {
                                let _ = data.next().ok_or(GraphSetParseError {
                                    message: "Missing '#' in graph".to_string(),
                                })?;
                                let id = data.next().ok_or(GraphSetParseError {
                                    message: "Id for graph is missing".to_string(),
                                })?;
                                if id == "-1" {
                                    break;
                                }
                                if current_graph.id != usize::MAX {
                                    graph_list.push(current_graph);
                                }
                                let id = id.parse::<usize>();
                                match id {
                                    Ok(id) => {
                                        current_graph = Graph::new(id);
                                        if id != graph_id {
                                            return Err(GraphSetParseError {
                                                    message: format!("Graph with graph id {}, it should have the id {}", id, graph_id),
                                                });
                                        }
                                        graph_id += 1;
                                    }
                                    _ => {
                                        return Err(GraphSetParseError {
                                            message: "Id for graph invalid".to_string(),
                                        });
                                    }
                                }
                            }
                            "v" => {
                                let id = data.next().ok_or(GraphSetParseError {
                                    message: format!(
                                        "Graph {}, Missing id for a vertex in",
                                        current_graph.id
                                    )
                                    .to_string(),
                                })?;
                                let id = id.parse::<usize>();
                                match id {
                                    Ok(id) => {
                                        let vertex = current_graph.create_vertex();
                                        if vertex.id != id {
                                            return Err(GraphSetParseError {
                                                    message: format!("Graph {}, Vertex ID ({}) in input file does not fit the expected ID {}", current_graph.id.clone(), id, current_graph.get_last_vertex().id)
                                                });
                                        }
                                        let label = data.next().ok_or(GraphSetParseError {
                                            message: format!(
                                                "Graph {}, Missing label for a vertex",
                                                current_graph.id
                                            )
                                            .to_string(),
                                        })?;
                                        let label = label.parse::<usize>();
                                        if label.is_err() {
                                            return Err(GraphSetParseError {
                                                message: format!(
                                                    "Graph {}, Vertex {}, Label invalid",
                                                    current_graph.id, id
                                                ),
                                            });
                                        }
                                        current_graph.get_last_vertex().vertex_type =
                                            label.unwrap();
                                        let vertex_type =
                                            data.next().ok_or(GraphSetParseError {
                                                message: format!(
                                                    "Graph {}, Missing vertex type for a vertex",
                                                    current_graph.id
                                                )
                                                .to_string(),
                                            })?;
                                        let vertex_type = vertex_type.parse::<usize>();
                                        if vertex_type.is_err() {
                                            return Err(GraphSetParseError {
                                                message: format!(
                                                    "Graph {}, Vertex {}, Vertex type is invalid",
                                                    current_graph.id, id
                                                ),
                                            });
                                        }
                                        current_graph.get_last_vertex().vertex_type =
                                            vertex_type.unwrap();
                                    }
                                    _ => {
                                        return Err(GraphSetParseError {
                                            message: format!(
                                                "Graph {}, Vertex ID invalid",
                                                current_graph.id
                                            )
                                            .to_string(),
                                        });
                                    }
                                }
                            }
                            "e" => {
                                let from_id = data.next().ok_or(GraphSetParseError {
                                    message: format!(
                                        "Graph {}, Missing from id for an edge",
                                        current_graph.id
                                    )
                                    .to_string(),
                                })?;
                                let from_id: usize = match from_id.parse() {
                                    Ok(value) => value,
                                    _ => {
                                        return Err(GraphSetParseError {
                                            message: format!(
                                                "Graph {}, Invalid from id for an edge",
                                                current_graph.id
                                            )
                                            .to_string(),
                                        });
                                    }
                                };
                                let to_id = data.next().ok_or(GraphSetParseError {
                                    message: format!(
                                        "Graph {}, Missing to id for a edge in",
                                        current_graph.id
                                    )
                                    .to_string(),
                                })?;
                                let to_id: usize = match to_id.parse() {
                                    Ok(value) => value,
                                    _ => {
                                        return Err(GraphSetParseError {
                                            message: format!(
                                                "Graph {}, Invalid to id for a edge",
                                                current_graph.id
                                            )
                                            .to_string(),
                                        });
                                    }
                                };
                                let e_label = data.next().ok_or(GraphSetParseError {
                                    message: format!(
                                        "Graph {}, Missing edge label for a edge",
                                        current_graph.id
                                    )
                                    .to_string(),
                                })?;
                                let e_label: usize = match e_label.parse() {
                                    Ok(value) => value,
                                    _ => {
                                        return Err(GraphSetParseError {
                                            message: format!(
                                                "Graph {}, Invalid e_label for a edge",
                                                current_graph.id
                                            )
                                            .to_string(),
                                        });
                                    }
                                };

                                if !current_graph.has_vertex_with_id(&from_id)
                                    || !current_graph.has_vertex_with_id(&to_id)
                                {
                                    return Err(GraphSetParseError {
                                        message: format!(
                                            "Graph {}, Edge invalid, ids of vertices not found",
                                            current_graph.id
                                        )
                                        .to_string(),
                                    });
                                }

                                let from_vertex: Option<&mut Vertex> =
                                    current_graph.vertices.get_mut(from_id);
                                match from_vertex {
                                    Some(from_vertex) => {
                                        from_vertex.push(to_id, e_label);
                                    }
                                    _ => {
                                        return Err(GraphSetParseError {
                                            message: format!(
                                                "Graph {}, Edge invalid, ids of vertices not found",
                                                current_graph.id
                                            )
                                            .to_string(),
                                        })
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            Err(_) => {
                return Err(GraphSetParseError {
                    message: "Error reading file".to_string(),
                });
            }
        }
        if current_graph.id != usize::MAX {
            graph_list.push(current_graph);
        }
        Ok(graph_list)
    }

    pub fn to_str_repr(&self, support: Option<usize>) -> String {
        let mut lines: Vec<String> = Vec::new();
        let mut g_rep = format!("t # {}", self.id);
        if let Some(support) = support {
            g_rep += &*format!(" * {}", support);
        }
        lines.push(g_rep);
        let mut edges: Vec<&Edge> = Vec::new();
        for vertex in &self.vertices {
            lines.push(vertex.to_str_repr());
            edges.extend(vertex.edges.iter());
        }
        for edge in edges {
            lines.push(edge.to_str_repr());
        }
        lines.join("\n")
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_simple_graph() {
        let mut graph = Graph::new(1);
        graph.create_vertex_with_data(1, 2);
        graph.create_vertex_with_data(2, 2);
        graph.create_vertex_with_data(3, 4);
        graph.vertices.get_mut(0).unwrap().push(1, 0);
        graph.vertices.get_mut(0).unwrap().push(2, 0);
        graph.vertices.get_mut(1).unwrap().push(2, 0);
        println!("{:?}", graph);
        assert_eq!(graph.vertices.len(), 3);
        assert_eq!(graph.vertices.first().unwrap().edges.len(), 2);
        assert_eq!(graph.vertices.get(1).unwrap().edges.len(), 1);
        assert_eq!(graph.vertices.get(2).unwrap().edges.len(), 0);
    }

    #[test]
    fn test_load_graphs_from_file() {
        let _now = Instant::now();
        let graphs = Graph::graphs_set_from_file("graphs.txt");
        match graphs {
            Ok(ref graphs) => {
                println!("All good parsing input file, found {} graphs", graphs.len());
            }
            Err(err) => panic!("{}", err.to_string()),
        }
        let _graphs = graphs.unwrap();
        let delta = _now.elapsed().as_millis();
        println!("Took {}ms", delta);
    }
}
