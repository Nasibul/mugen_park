use chrono::prelude::*;
use polars::prelude::*;

/// Reads a CSV file into a DataFrame.
///
/// This function reads the CSV file located at the specified `path` and returns
/// the resulting DataFrame. It infers the schema and assumes the CSV file has a header.
///
/// # Arguments
///
/// * `path` - A string slice representing the file path to the CSV file.
///
/// # Returns
///
/// * A `Result` containing the DataFrame or a `PolarsError`.
///
/// # Errors
///
/// This function will return an error if the CSV file cannot be read or parsed.
///
/// # Examples
///
/// ```
/// use polars::prelude::*;
///
/// let df = read_csv_to_df("data.csv").expect("Failed to read CSV file");
/// ```
pub(crate) fn read_csv_to_df(path: &str) -> Result<DataFrame, PolarsError> {
    let df: DataFrame = CsvReader::from_path(path)?
        .infer_schema(None)
        .has_header(true)
        .finish()?;
    Ok(df)
}


/// Reads multiple CSV files into a single DataFrame by vertically stacking them.
///
/// This function reads each CSV file specified in the `paths` vector, and vertically
/// stacks them into a single DataFrame.
///
/// # Arguments
///
/// * `paths` - A vector of string slices representing the file paths to the CSV files.
///
/// # Returns
///
/// * A `Result` containing the combined DataFrame or a `PolarsError`.
///
/// # Errors
///
/// This function will return an error if any CSV file cannot be read or parsed, or if
/// vertical stacking of DataFrames fails.
///
/// # Examples
///
/// ```
/// use polars::prelude::*;
///
/// let paths = vec!["data1.csv", "data2.csv"];
/// let combined_df = read_multiple_csvs(paths).expect("Failed to read and combine CSV files");
/// ```
pub(crate) fn read_multiple_csvs(paths: Vec<&str>) -> Result<DataFrame, PolarsError> {
    let mut dfs: DataFrame = DataFrame::default();

    for path in paths {
        let df: DataFrame = read_csv_to_df(path)?;
        dfs.vstack_mut(&df)?;
    }
    Ok(dfs)
}

/// Converts a date string to a `NaiveDateTime`.
///
/// # Arguments
///
/// * `date_str` - The date string to convert.
/// * `format` - The format of the date string.
///
/// # Returns
///
/// * A `Result` containing the `NaiveDateTime` or a `chrono::ParseError`.
///
/// # Errors
///
/// This function will return a `chrono::ParseError` if the date string does not match the format.
pub(crate) fn str_to_datetime(str_val: &Series, format: &str) -> Series {
    let datetime_result: Vec<NaiveDateTime> = str_val
        .str()
        .unwrap()
        .into_iter()
        .filter_map(|s| NaiveDateTime::parse_from_str(s.unwrap(), format).ok())
        .collect::<Vec<chrono::NaiveDateTime>>();

    let datetime_chunked: Logical<DatetimeType, Int64Type> =
        DatetimeChunked::from_naive_datetime("timestamp", datetime_result, TimeUnit::Milliseconds);
    datetime_chunked.into_series()
}

/// Processes the ground truth DataFrame by filtering and transforming columns.
///
/// This function filters the `ground_truth` DataFrame to include only rows where the "Name"
/// column is the argument given. It then drops the columns "Time Zone", "Name", and "PTID", and converts
/// the "Time Stamp" column from a string to a `NaiveDateTime`.
///
/// # Arguments
///
/// * `ground_truth` - The input DataFrame containing the ground truth data.
///
/// # Returns
///
/// * A `Result` containing the processed DataFrame or an error.
///
/// # Errors
///
/// This function will return an error if filtering or column transformation fails.
pub(crate) fn process_truth(ground_truth: DataFrame, region: &str) -> Result<DataFrame, PolarsError>{
    let mut ground_truth_filtered: DataFrame = ground_truth
        .filter(&ground_truth["Name"].equal(region)?)?
        .drop_many(&vec!["Time Zone", "Name", "PTID"]);
    ground_truth_filtered.apply("Time Stamp", |s| str_to_datetime(s, "%m/%d/%Y %H:%M:%S"))?;
    Ok(ground_truth_filtered)
}

/// Processes the prediction DataFrame by filtering and transforming columns.
///
/// This function filters the `pred` DataFrame to include only the "Time Stamp" column
/// and the specified `region` column. It then converts the "Time Stamp" column from a
/// string to a `NaiveDateTime`.
///
/// # Arguments
///
/// * `pred` - The input DataFrame containing the prediction data.
/// * `region` - A string slice specifying the region column to be included.
///
/// # Returns
///
/// * A `Result` containing the processed DataFrame or a `PolarsError`.
///
/// # Errors
///
/// This function will return an error if the selection or column transformation fails.
///
/// # Examples
///
/// ```
/// use polars::prelude::*;
///
/// // Assuming you have a DataFrame `df` and a region name "Region1"
/// let processed_df = process_pred(df, "Region1").expect("Processing failed");
/// ```
pub(crate) fn process_pred(pred: DataFrame, region: &str) -> Result<DataFrame, PolarsError>{
    let mut pred_filtered = pred.select(&vec!["Time Stamp", region])?;
    pred_filtered.apply("Time Stamp", |s| str_to_datetime(s, "%m/%d/%Y %H:%M"))?;
    Ok(pred_filtered)
}