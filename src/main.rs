use crate::data::graph::Graph;
use std::time::Instant;

pub mod cpd;
pub mod data;

use clap::Parser;
use cpd::{
    candidate_generation::AlgoCandidateGeneration,
    config::CPDConfig,
    graph_matching::{AlgoGraphMatching, GEDEditCosts},
};

/// Fast Rust implementation for Collaboration Pattern Discovery
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file with the graph database
    #[arg(short, long)]
    input: String,

    /// Output file for the resulting subgraphs, if "sdtout", the resulting patterns will be printed to the
    /// console after processing finished with ######
    #[arg(short, long, default_value = "stdout")]
    output: String,

    /// Exact support
    #[arg(long, default_value_t = 2)]
    support_exact: usize,

    /// Relaxed support
    #[arg(long, default_value_t = 2)]
    support_relaxed: usize,

    /// Graph matching:
    /// - "cosine" (node and edge vector similarity, uses the alpha parameter),
    /// - "ged" (approx. graph edit distance)
    /// - "vf2" (only exact matches)
    #[arg(long, default_value = "cosine")]
    graph_matching: String,

    /// Relaxed threshold
    /// - values [0.0..1.0] for graph matching "cosine" (1.0 means exact matches)
    /// - values >= 0 for graph matching "ged" (0 means exact matches)
    #[arg(long, default_value_t = 0.95)]
    relaxed_threshold: f64,

    /// Activity vertex type
    #[arg(long, default_value_t = 0)]
    activity_vertex_type: usize,

    /// Object vertex types
    #[arg(long, num_args = 0..)]
    object_vertex_types: Vec<usize>,

    /// Minimum number of main vertices
    #[arg(long, default_value_t = 4)]
    min_vertices: usize,

    /// Maximum number of the main vertices
    #[arg(long, default_value_t = 5)]
    max_vertices: usize,

    /// The alpha value between 0.0 and 1.0 defines the weight importance of the vertex and edge
    /// vectors: if 1.0, the edges are ignored; if 0.0, the vertices are ignored
    #[arg(long, default_value_t = 0.5)]
    alpha: f64,

    /// Supress debug statements
    #[arg(long, default_value_t = false)]
    silence: bool,
}

fn main() {
    let args = Args::parse();
    let silence = args.silence;

    if !silence {
        println!("-----------------------");
        println!("| CPD Subgraph Mining |");
        println!("-----------------------");
    }
    if args.min_vertices > args.max_vertices {
        eprintln!(
            "Parameter error! Min number of activity vertices ({}) > Max number of activity vertices ({})!",
            args.min_vertices, args.max_vertices
        );
        return;
    }
    if args.graph_matching == "cosine" && (args.alpha > 1.0 || args.alpha < 0.0) {
        eprintln!(
            "Parameter error! --alpha should be 0.0 <= alpha <= 1.0, is {}",
            args.alpha
        );
        return;
    }
    if args.graph_matching == "cosine"
        && (args.relaxed_threshold > 1.0 || args.relaxed_threshold < 0.0)
    {
        eprintln!(
            "Parameter error! for cosine graph matchting, the --relaxed-threshold should be 0.0 <= relaxed_threshold <= 1.0, is {}",
            args.relaxed_threshold
        );
        return;
    }
    if args.graph_matching == "ged" && args.relaxed_threshold < 0.0 {
        eprintln!(
            "Parameter error! for ged graph matchting, the --relaxed-threshold should be >= 0, is {}",
            args.relaxed_threshold
        );
        return;
    }
    let now = Instant::now();
    let graphs = Graph::graphs_set_from_file(args.input);
    let graphs = match graphs {
        Ok(ref graphs) => {
            if !silence {
                println!("All good parsing input file, found {} graphs", graphs.len());
            }
            graphs
        }
        Err(err) => {
            eprintln!("Error parsing input file: {}", err);
            return;
        }
    };
    let mut graph_matching = AlgoGraphMatching::GEDFastHungarian {
        edit_costs: GEDEditCosts::default(),
        matching_threshold: args.relaxed_threshold.round() as usize,
    };
    if args.graph_matching == "cosine" {
        graph_matching = AlgoGraphMatching::CosineSimilarity {
            alpha: args.alpha,
            matching_threshold: args.relaxed_threshold,
        }
    }
    if args.graph_matching == "vf2" {
        graph_matching = AlgoGraphMatching::VF2IsomorphismTest;
    }
    let cpd_config = CPDConfig::new(
        AlgoCandidateGeneration::FullyConnected {
            activity_vertex_type: args.activity_vertex_type,
            object_vertex_types: args.object_vertex_types,
            min_number_of_activity_vertices: args.min_vertices,
            max_number_of_activity_vertices: args.max_vertices,
        },
        graph_matching,
        args.support_exact,
        args.support_relaxed,
        args.silence,
    );
    if !silence {
        println!("Mining patterns..");
    };
    let patterns = cpd_config.run(graphs);
    let delta = now.elapsed().as_millis();
    if !silence {
        println!("Finished. Total time: {delta}ms");
        println!("#######");
    };
    if args.output == "stdout" {
        for g in patterns.iter() {
            println!(
                "{}",
                g.pattern
                    .to_str_repr(Some(g.frequency_exact), Some(g.frequency_relaxed))
            );
        }
    } else {
        use std::fs::File;
        use std::io::Write;

        let file = File::create(&args.output);
        if file.is_err() {
            eprintln!("Failed to create output file");
            return;
        }
        let mut file = file.unwrap();

        for g in patterns.iter() {
            let line = format!(
                "{}\n",
                g.pattern
                    .to_str_repr(Some(g.frequency_exact), Some(g.frequency_relaxed))
            );
            file.write_all(line.as_bytes())
                .expect("Failed to write to output file");
        }
        if !silence {
            println!("Result exported to {}", args.output);
        }
    }
}
