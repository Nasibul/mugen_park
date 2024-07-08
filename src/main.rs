use polars::prelude::*;
mod data;
use data::{process_pred, process_truth, read_csv_to_df, read_multiple_csvs};
mod graph;
use crate::graph::Graph;
use graph::{GraphConfig, LineGraph, PieGraph};
fn main() -> Result<(), PolarsError> {
    let ground_truth_data_paths: Vec<&str> = vec![
        "data/20231201palIntegrated.csv",
        "data/20231202palIntegrated.csv",
        "data/20231203palIntegrated.csv",
        "data/20231204palIntegrated.csv",
        "data/20231205palIntegrated.csv",
        "data/20231206palIntegrated.csv",
        "data/20231207palIntegrated.csv",
        "data/20231208palIntegrated.csv",
        "data/20231209palIntegrated.csv",
        "data/20231210palIntegrated.csv",
    ];
    let ground_truth: DataFrame =
        process_truth(read_multiple_csvs(ground_truth_data_paths)?, "N.Y.C.")?;

    let predictions: DataFrame = process_pred(read_csv_to_df("data/20231209isolf.csv")?, "N.Y.C.")?;

    let config = GraphConfig {
        title: "Ground Truth VS Predictions for NYC",
        x_label: "Time",
        y_label : "Megawatts",
        ..Default::default()
    };

    let line_graph: LineGraph = LineGraph {
        config: config,
        data: ground_truth,
        forecast: predictions,
        ..Default::default()
        };
    line_graph.draw();

    Ok(())
}
