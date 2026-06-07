use std::{
    collections::{HashMap, HashSet},
    env, error,
    fs::File,
    process::exit,
};

#[derive(Debug)]
enum OutputType {
    HumanReadable,
    Json,
    Csv,
}

fn get_header_indices(file: &mut csv::Reader<File>, column_names: &Vec<&str>) -> Vec<usize> {
    let mut column_indices: Vec<usize> = Vec::new();
    let headers: Vec<&str> = match file.headers() {
        Ok(h) => Vec::from_iter(h.iter()),
        Err(_) => Vec::new(),
    };
    for column_name in column_names {
        match headers.iter().position(|r| r == column_name) {
            Some(index) => column_indices.push(index),
            None => (),
        };
    }
    column_indices
}

fn populate_file_map(
    file: &mut csv::Reader<File>,
    key_column_indices: &Vec<usize>,
) -> HashMap<Vec<String>, Vec<String>> {
    let mut file_map: HashMap<Vec<String>, Vec<String>> = HashMap::new();
    let mut key: Vec<String>;
    let mut record: Vec<String>;
    for result in file.records() {
        match result {
            Ok(_record) => {
                record = Vec::from_iter(_record.iter().map(|x| x.to_string()));
                key = Vec::new();
                for index in key_column_indices {
                    key.push(record[*index].clone());
                }
                file_map.insert(key, record);
            }
            Err(_) => (),
        };
    }
    file_map
}

fn print_diff(
    only_base: &Vec<&Vec<String>>,
    only_new: &Vec<&Vec<String>>,
    different: &HashSet<(&Vec<String>, &str, String, String)>,
) {
    if only_base.is_empty() {
        println!("No records present which exist only in the base file.");
    } else {
        println!("Records that exist only in the base file:");
        for value in only_base {
            println!("{value:?}");
        }
    }

    if only_new.is_empty() {
        println!("No records present which exist only in the new file.");
    } else {
        println!("Records that exist only in the new file:");
        for value in only_new {
            println!("{value:?}");
        }
    }

    if different.is_empty() {
        println!(
            "No different records found in the same-keyed records between the base and new files."
        );
    } else {
        for (key, column_name, base_value, new_value) in different.iter() {
            println!(
                "For record {:?}, column \"{}\", base has \"{}\" and new has \"{}\".",
                key, column_name, base_value, new_value
            );
        }
    }
}

fn diff_csv(
    base_file: &mut csv::Reader<File>,
    new_file: &mut csv::Reader<File>,
    key_columns: &Vec<&str>,
) {
    let base_file_headers = base_file.headers().unwrap().clone();
    let new_file_headers = new_file.headers().unwrap().clone();

    let base_file_key_indices = get_header_indices(base_file, key_columns);
    let new_file_key_indices = get_header_indices(new_file, key_columns);

    let base_file_map: HashMap<Vec<String>, Vec<String>> =
        populate_file_map(base_file, &base_file_key_indices);
    let new_file_map: HashMap<Vec<String>, Vec<String>> =
        populate_file_map(new_file, &new_file_key_indices);

    let mut in_base_file: Vec<&Vec<String>> = Vec::new();
    let mut in_new_file: Vec<&Vec<String>> = Vec::new();
    let mut different_records: HashSet<(&Vec<String>, &str, String, String)> = HashSet::new();

    for base_key in base_file_map.keys() {
        if !new_file_map.contains_key(base_key) {
            in_base_file.push(base_file_map.get(base_key).unwrap());
            continue;
        }
        let base_value = base_file_map.get(base_key).unwrap();
        let new_value = new_file_map.get(base_key).unwrap();
        if base_value == new_value {
            continue;
        }
        for index in 0..base_value.len() {
            if base_value.get(index) == new_value.get(index) {
                continue;
            } else {
                different_records.insert((
                    base_key,
                    base_file_headers.get(index).unwrap_or(""),
                    base_value.get(index).unwrap_or(&String::new()).clone(),
                    new_value.get(index).unwrap_or(&String::new()).clone(),
                ));
            }
        }
    }

    for new_key in new_file_map.keys() {
        if !base_file_map.contains_key(new_key) {
            in_new_file.push(new_file_map.get(new_key).unwrap());
            continue;
        }
        let base_value = base_file_map.get(new_key).unwrap();
        let new_value = new_file_map.get(new_key).unwrap();
        if base_value == new_value {
            continue;
        }
        for index in 0..base_value.len() {
            if base_value.get(index) == new_value.get(index) {
                continue;
            } else {
                different_records.insert((
                    new_key,
                    new_file_headers.get(index).unwrap_or(""),
                    base_value.get(index).unwrap_or(&String::new()).clone(),
                    new_value.get(index).unwrap_or(&String::new()).clone(),
                ));
            }
        }
    }

    print_diff(&in_base_file, &in_new_file, &different_records);
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut base_file_path = "";
    let mut new_file_path = "";
    let mut debug = false;
    let mut output_type = OutputType::HumanReadable;
    let mut key_columns: Vec<&str> = Vec::new();

    let mut arg_index = 1; // Ignoring argv[0], the program name.
    let args: Vec<String> = env::args().collect();
    let args_count = args.len();
    while arg_index < args_count {
        match args[arg_index].as_str() {
            "--debug" => debug = true,
            "--base" => {
                if arg_index + 1 >= args_count {
                    println!("Did not provide an argument for --base.");
                    exit(1);
                }

                arg_index += 1;
                base_file_path = args[arg_index].as_str();
            }
            "--new" => {
                if arg_index + 1 >= args_count {
                    println!("Did not provide an argument for --new.");
                    exit(1);
                }

                arg_index += 1;
                new_file_path = args[arg_index].as_str();
            }
            "--output" => {
                if arg_index + 1 >= args_count {
                    println!("Did not provide an argument for --new.");
                    exit(1);
                }

                arg_index += 1;
                output_type = match args[arg_index].to_lowercase().as_str() {
                    "human-readable" => OutputType::HumanReadable,
                    "human" => OutputType::HumanReadable,
                    "json" => OutputType::Json,
                    "csv" => OutputType::Csv,
                    unmatched => {
                        println!("Unrecognized output type {unmatched}. Defaulting to JSON.");
                        OutputType::Json
                    }
                }
            }
            "--keys" => {
                if arg_index + 1 >= args_count {
                    println!("Did not provide an argument for --keys.");
                    exit(1);
                }

                arg_index += 1;
                key_columns = args[arg_index].split(',').collect();
            }
            unmatched => {
                if debug {
                    println!("WARN: Unmatched option {unmatched}.")
                }
            }
        }
        arg_index += 1;
    }

    // Base file setup.
    if base_file_path.is_empty() {
        println!("Did not provide an argument for --base.");
        exit(1);
    }
    if debug {
        println!("Base file path is {base_file_path}.");
    }
    let mut base_file = match csv::ReaderBuilder::new().from_path(base_file_path) {
        Ok(reader) => reader,
        Err(err) => {
            println!("Could not initialize base file reader: {err}.");
            exit(1);
        }
    };
    if debug {
        println!("Base file reader: {base_file:?}");
    }

    // New file setup.
    if new_file_path.is_empty() {
        println!("Did not provide an argument for --new.");
        exit(1);
    }
    if debug {
        println!("New file path is {new_file_path}.");
    }
    let mut new_file = match csv::ReaderBuilder::new().from_path(new_file_path) {
        Ok(reader) => reader,
        Err(err) => {
            println!("Could not initialize new file reader: {err}.");
            exit(1);
        }
    };
    if debug {
        println!("New file reader: {new_file:?}");
    }

    if debug {
        println!("Output type set to {output_type:?}.");
    }

    let _ = diff_csv(&mut base_file, &mut new_file, &key_columns);

    Ok(())
}
