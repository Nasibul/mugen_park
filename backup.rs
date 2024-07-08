use chrono::prelude::*;
use plotters::coord::types::RangedCoordf64;
use plotters::prelude::*;
use polars::prelude::*;
use std::iter::zip;

fn read_csv_to_df(path: &str) -> Result<DataFrame, PolarsError> {
    let df = CsvReader::from_path(path)?
        .infer_schema(None)
        .has_header(true)
        .finish()?;
    Ok(df)
}

fn read_multiple_csvs(paths: Vec<&str>) -> Result<DataFrame, PolarsError> {
    let mut dfs = DataFrame::default();

    for path in paths {
        let df = read_csv_to_df(path)?;
        dfs.vstack_mut(&df)?;
    }
    Ok(dfs)
}

fn str_to_datetime(str_val: &Series, format: &str) -> Series {
    let datetime_result = str_val
        .str()
        .unwrap()
        .into_iter()
        .filter_map(|s| NaiveDateTime::parse_from_str(s.unwrap(), format).ok())
        .collect::<Vec<chrono::NaiveDateTime>>();

    let datetime_chunked =
        DatetimeChunked::from_naive_datetime("timestamp", datetime_result, TimeUnit::Milliseconds);
    datetime_chunked.into_series()
}

fn main() -> Result<(), PolarsError> {
    let ground_truth_data_paths = vec![
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
    let ground_truth = read_multiple_csvs(ground_truth_data_paths)?;
    let mut ground_truth_filtered = ground_truth
        .filter(&ground_truth["Name"].equal("N.Y.C.")?)?
        .drop_many(&vec!["Time Zone", "Name", "PTID"]);

    let mut predictions =
        read_csv_to_df("data/20231209isolf.csv")?.select(&vec!["Time Stamp", "N.Y.C."])?;

    ground_truth_filtered.apply("Time Stamp", |s| str_to_datetime(s, "%m/%d/%Y %H:%M:%S"))?;
    predictions.apply("Time Stamp", |s| str_to_datetime(s, "%m/%d/%Y %H:%M"))?;

    let now = Local::now().format("%Y%m%d_%H:%M:%S").to_string();
    let filename = format!("plot_{}.png", now);
    let root = BitMapBackend::new(&filename, (3840, 2160)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let min_date =
        NaiveDateTime::from(&ground_truth_filtered.column("Time Stamp")?.get(0).unwrap());
    let max_date = NaiveDateTime::from(
        &ground_truth_filtered
            .column("Time Stamp")?
            .get(ground_truth_filtered.height() - 1)
            .unwrap(),
    );

    let mut chart = ChartBuilder::on(&root)
        .margin(30)
        .caption("Ground Truth VS Predictions for NYC", ("Arial", 80))
        .x_label_area_size(20)
        .y_label_area_size(70)
        .build_cartesian_2d(
            RangedDateTime::from(min_date..max_date),
            RangedCoordf64::from(4000.0..6500.0),
        )
        .unwrap();

    let x1 = ground_truth_filtered
        .column("Time Stamp")?
        .datetime()?
        .into_no_null_iter()
        .map(|dt| {
            DateTime::from_timestamp(dt / 1000, 0)
                .unwrap()
                .naive_local()
        })
        .collect::<Vec<NaiveDateTime>>();
    let y1 = ground_truth_filtered
        .column("Integrated Load")?
        .rechunk()
        .f64()?
        .into_no_null_iter()
        .collect::<Vec<f64>>();

    chart
        .configure_mesh()
        .label_style(("Calibri", 30))
        .x_label_formatter(&|x| x.format("%m/%d/%Y").to_string())
        .draw()
        .unwrap();
    chart
        .draw_series(LineSeries::new(
            zip(x1.iter(), y1.iter())
                .map(|(dt, value)| (*dt, *value))
                .collect::<Vec<(NaiveDateTime, f64)>>(),
            &BLACK,
        ))
        .unwrap()
        .label("Truth")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 80, y)], &BLACK));

    let x2 = predictions
        .column("Time Stamp")?
        .datetime()?
        .into_no_null_iter()
        .map(|dt| {
            DateTime::from_timestamp(dt / 1000, 0)
                .unwrap()
                .naive_local()
        })
        .collect::<Vec<NaiveDateTime>>();
    let y2 = predictions
        .column("N.Y.C.")?
        .rechunk()
        .i64()?
        .into_no_null_iter()
        .map(|x| x as f64)
        .collect::<Vec<f64>>();
    chart
        .draw_series(LineSeries::new(
            zip(x2.iter(), y2.iter())
                .map(|(dt, value)| (*dt, *value))
                .collect::<Vec<(NaiveDateTime, f64)>>(),
            *&ShapeStyle {
                color: GREEN.to_rgba(),
                filled: true,
                stroke_width: 3,
            },
        ))
        .unwrap()
        .label("Pred")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 80, y)], &GREEN));

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .margin(20)
        .legend_area_size(100)
        .label_font(("Calibri", 50))
        .border_style(&BLACK)
        .background_style(&WHITE.mix(0.8))
        .draw()
        .unwrap();

    root.present();
    Ok(())
}
