use crate::data::graph::Graph;
use core::f32;
use std::time::Instant;

pub mod cpd;
pub mod data;

use clap::Parser;
use cpd::{
    candidate_generation::AlgoCandidateGeneration, config::CPDConfig,
    graph_matching::AlgoGraphMatching,
};

/// Fast Rust implementation for gSpan
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

    /// Min exact support
    #[arg(long, default_value_t = 2)]
    support_exact: usize,

    /// Min relaxed support
    #[arg(long, default_value_t = 2)]
    support_relaxed: usize,

    /// Relaxed threshold
    #[arg(long, default_value_t = 0.0f32)]
    relaxed_threshold: f32,

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
}

fn main() {
    let args = Args::parse();

    println!("-----------------------");
    println!("| CPD Subgraph Mining |");
    println!("-----------------------");
    // println!("Using arguments:");
    // println!("{:?}", args);
    let now = Instant::now();
    let graphs = Graph::graphs_set_from_file(args.input);
    match graphs {
        Ok(ref graphs) => {
            println!("All good parsing input file, found {} graphs", graphs.len());
        }
        Err(err) => panic!("{}", err.to_string()),
    }
    let graphs = graphs.unwrap();
    let cpd_config = CPDConfig::new(
        AlgoCandidateGeneration::FullyConnected {
            activity_vertex_type: args.activity_vertex_type,
            object_vertex_types: args.object_vertex_types,
            min_number_of_activity_vertices: args.min_vertices,
            max_number_of_activity_vertices: args.max_vertices,
        },
        AlgoGraphMatching::CosineSimilarity {
            alpha: 0.5f32,
            matching_threshold: args.relaxed_threshold,
        },
        args.support_exact,
        args.support_relaxed,
    );
    println!("Mining patterns..");
    let patterns = cpd_config.run(&graphs);
    let delta = now.elapsed().as_millis();
    println!(
        "Finished. Final patterns: {}; took {delta}ms",
        patterns.len()
    );
    println!("#######");
    if args.output == "stdout" {
        for g in patterns.iter() {
            println!(
                "{}",
                g.pattern
                    .to_str_repr(Some(g.frequency_exact), Some(g.frequency_relaxed))
            );
        }
    } else {
    }
}
