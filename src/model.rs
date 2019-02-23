use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use rulinalg::vector::Vector;

#[derive(Debug)]
pub struct Model(pub HashMap<String, Vector<f64>>);

pub fn load_model(filename: &str) -> Model {
    let mut vec_size = 100;
    let mut model = HashMap::new();
    let reader = BufReader::new(File::open(filename).expect("model file could not be opened"));
    for (i, maybe_line) in reader.lines().enumerate() {
        if let Ok(line) = maybe_line {
            let fields: Vec<&str> = line.trim().split(' ').collect();
            if fields.len() == 2 {
                vec_size = fields[1].parse().expect("invalid vector size");
            } else if fields.len() == vec_size + 1 {
                let word = fields[0].to_string();
                let vector = Vector::from_fn(vec_size, |i| fields[i+1].parse().unwrap());
                model.insert(word, vector);
            } else {
                eprintln!("could not parse line {} with {} fields", i, fields.len());
            }
        }
    }
    Model(model)
}