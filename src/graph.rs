use charming::{
    component::{Axis, Grid, Legend, Title},
    element::{AxisLabel, AxisType, LineStyle, TextStyle},
    renderer::image_renderer::ImageRenderer,
    series::Line,
    theme::Theme,
    Chart, ImageFormat,
};
use chrono::{DateTime, Utc};
use polars::prelude::*;
use std::default::Default;

pub(crate) trait Graph {
    fn generate_filename(&self, graph_type: &str) -> String {
        let now: DateTime<Utc> = Utc::now();
        format!("charts/{}_{}.png", graph_type, now.format("%Y%m%d%H%M%S"))
    }

    fn draw(&self);
}

pub(crate) struct GraphConfig<'a> {
    pub(crate) title: &'a str,
    pub(crate) x_label: &'a str,
    pub(crate) y_label: &'a str,
    pub(crate) graph_width: u32,
    pub(crate) graph_height: u32,
}

impl Default for GraphConfig<'_> {
    fn default() -> Self {
        GraphConfig {
            title: "Graph",
            x_label: "X-axis",
            y_label: "Y-axis",
            graph_width: 3840,
            graph_height: 2160,
        }
    }
}

pub(crate) struct LineGraph<'a> {
    pub(crate) config: GraphConfig<'a>,
    pub(crate) data: DataFrame,
    pub(crate) notes: &'a str,
    pub(crate) forecast: DataFrame,
    pub(crate) line_thickness: u32,
    pub(crate) forecast_color: &'a str,
}

impl Default for LineGraph<'_> {
    fn default() -> Self {
        LineGraph {
            config: GraphConfig::default(),
            data: DataFrame::default(),
            notes: "",
            forecast: DataFrame::default(),
            line_thickness: 5,
            forecast_color: "GREEN",
        }
    }
}

impl Graph for LineGraph<'_> {
    fn draw(&self) {
        let filename = self.generate_filename("LineChart");

        let full_data = self
            .data
            .outer_join(&self.forecast, ["Time Stamp"], ["Time Stamp"])
            .unwrap()
            .sort(["Time Stamp_right"], SortMultipleOptions::default())
            .unwrap();

        let x_axis_data = self
            .data
            .column("Time Stamp")
            .unwrap()
            .clone()
            .append(self.forecast.column("Time Stamp").unwrap())
            .unwrap()
            .unique()
            .unwrap()
            .sort(SortOptions::default())
            .unwrap()
            .datetime()
            .unwrap()
            .into_no_null_iter()
            .map(|dt| {
                DateTime::from_timestamp(dt / 1000, 0)
                    .unwrap()
                    .naive_local()
                    .format("%m/%d/%Y %H:%M")
                    .to_string()
            })
            .collect::<Vec<String>>();

        let min_y = self
            .data
            .column("Integrated Load")
            .unwrap()
            .f64()
            .unwrap()
            .min()
            .unwrap();

        let max_y = self
            .data
            .column("Integrated Load")
            .unwrap()
            .f64()
            .unwrap()
            .max()
            .unwrap();

        let chart = Chart::new()
            .title(
                Title::new()
                    .text(self.config.title)
                    .text_style(TextStyle::new().font_size(100))
                    .left("center"),
            )
            .grid(
                Grid::new()
                    .left("4%")
                    .right("5%")
                    .bottom("3%")
                    .top("5%")
                    .contain_label(true),
            )
            .grid(
                Grid::new()
                    .left("4%")
                    .right("5%")
                    .bottom("3%")
                    .top("5%")
                    .contain_label(true),
            )
            .x_axis(
                Axis::new()
                    .name(self.config.x_label)
                    .axis_label(AxisLabel::new().rotate(60).font_size(30))
                    .name_text_style(TextStyle::new().font_size(60))
                    .type_(AxisType::Category)
                    .data(x_axis_data),
            )
            .y_axis(
                Axis::new()
                    .name(self.config.y_label)
                    .name_gap(35)
                    .axis_label(AxisLabel::new().font_size(30))
                    .name_text_style(TextStyle::new().font_size(60))
                    .min((min_y / 100.0).floor() * 100.0)
                    .max((max_y / 100.0).ceil() * 100.0),
            )
            .series(
                Line::new()
                    .line_style(LineStyle::new().width(self.line_thickness))
                    .data(
                        full_data
                            .column("Integrated Load")
                            .unwrap()
                            .f64()
                            .unwrap()
                            .into_no_null_iter()
                            .collect::<Vec<f64>>(),
                    ),
            )
            .x_axis(
                Axis::new().show(false).grid_index(1).data(
                    full_data
                        .column("Time Stamp_right")
                        .unwrap()
                        .datetime()
                        .unwrap()
                        .into_no_null_iter()
                        .map(|dt| {
                            DateTime::from_timestamp(dt / 1000, 0)
                                .unwrap()
                                .naive_local()
                                .format("%m/%d/%Y %H:%M")
                                .to_string()
                        })
                        .collect::<Vec<String>>(),
                ),
            )
            .y_axis(
                Axis::new()
                    .show(false)
                    .grid_index(1)
                    .min((min_y / 100.0).floor() * 100.0)
                    .max((max_y / 100.0).ceil() * 100.0),
            )
            .series(
                Line::new()
                    .line_style(
                        LineStyle::new()
                            .width(self.line_thickness)
                            .color(self.forecast_color),
                    )
                    .data(
                        full_data
                            .column("N.Y.C.")
                            .unwrap()
                            .i64()
                            .unwrap()
                            .into_no_null_iter()
                            .collect::<Vec<i64>>(),
                    ),
            )
            .legend(
                Legend::new()
                    .left(50)
                    .top(50)
                    .data(vec!["Actual", "Forecast"]),
            );

        let mut renderer = ImageRenderer::new(self.config.graph_width, self.config.graph_height)
            .theme(Theme::Dark);
        renderer.render_format(ImageFormat::Png, &chart).unwrap();
        let _ = renderer.save_format(ImageFormat::Png, &chart, filename);
    }
}

pub(crate) struct PieGraph<'a> {
    pub(crate) config: GraphConfig<'a>,
    pub(crate) data: DataFrame,
    pub(crate) notes: str,
}
