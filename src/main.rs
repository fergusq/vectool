use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::{HashMap, binary_heap::BinaryHeap};
use std::cmp::Ordering;
use regex::Regex;
use rulinalg::vector::Vector;
use rulinalg::norm::Euclidean;

struct Model(HashMap<String, Vector<f64>>);

fn load_model(filename: &String) -> Model {
    let mut vec_size = 100;
    let mut model = HashMap::new();
    let reader = BufReader::new(File::open(filename).expect("file could not be opened"));
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

fn find_nn(model: &Model, word: String) -> String {
    if !model.0.contains_key(&word) {
        return word;
    }
    let vector = &model.0[&word];
    let mut nearest_word = &word;
    let mut nearest_distance = 0.0;
    for (w, v) in model.0.iter() {
        let distance = vector.dot(v) / v.norm(Euclidean);
        if !word.eq(w) && distance > nearest_distance {
            nearest_distance = distance;
            nearest_word = &w;
        }
    }
    nearest_word.clone()
}

#[derive(PartialOrd, PartialEq)]
struct NearWord(f64, String);

impl Eq for NearWord {}
impl Ord for NearWord {
    fn cmp(&self, other: &NearWord) -> Ordering {
        if self.0 < other.0 {
            Ordering::Less
        } else if self.0 > other.0 {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

fn find_nnk(model: &Model, vector: &Vector<f64>, k: usize) -> Vec<(f64, String)> {
    let mut heap = BinaryHeap::new();
    let norm = vector.norm(Euclidean);
    for (w, v) in model.0.iter() {
        let distance = vector.dot(v) / v.norm(Euclidean) / norm;
        heap.push(NearWord(distance, w.clone()));
    }
    let mut ans = Vec::new();
    for _ in 0..k {
        let NearWord(dis, word) = heap.pop().unwrap();
        ans.push((dis, word));
    }
    ans
}

fn filter(filename: &String) {
    let special_character_regex = Regex::new("[^a-zåäö ]").unwrap();
    let model = load_model(filename);
    eprintln!("Loaded {} word model", model.0.len());
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        println!("{}",
            special_character_regex.replace_all(
                line.unwrap()
                    .to_lowercase()
                    .as_str(),
                ""
            )
            .split(' ')
            .map(String::from)
            .map(|word| find_nn(&model, word))
            .collect::<Vec<String>>()
            .join(" "));
    }
}

fn nn(filename: &String) {
    let model = load_model(filename);
    eprintln!("Loaded {} word model", model.0.len());
    let stdin = std::io::stdin();
    'outer: for line in stdin.lock().lines() {
        let preprocessed_line = line.unwrap()
            .trim()
            .to_lowercase();
        
        let words = preprocessed_line
            .split(' ')
            .collect::<Vec<&str>>();
        
        if words.len() % 2 != 1 {
            eprintln!("wrong number of operands");
        }

        let first_word = words[0].to_string();
        if !model.0.contains_key(&first_word) {
            eprintln!("unknown word `{}'", first_word);
            continue;
        }

        let mut vector = model.0[&first_word].clone();
        for i in (1..words.len()).step_by(2) {
            let word = words[i+1].to_string();
            if !model.0.contains_key(&word) {
                eprintln!("unknown word `{}'", word);
                continue 'outer;
            }

            let vector2 = &model.0[&word];
            match words[i] {
                "+" => {
                    vector = vector + vector2;
                }
                "-" => {
                    vector = vector - vector2;
                }
                _ => {
                    eprintln!("unknown operator `{}'", words[i]);
                }
            }
        }
        for (distance, near_word) in find_nnk(&model, &vector, 10) {
            if words.contains(&near_word.as_str()) {
                println!("({} {:.4})", near_word, distance);
            } else {
                println!("{} {:.4}", near_word, distance);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        3 => {
            match args[1].as_str() {
                "filter" => filter(&args[2]),
                "nn" => nn(&args[2]),
                _ => {
                    eprintln!("unknown command: {}", args[1]);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("usage: vectool (filter|nn) <model>");
            std::process::exit(1);
        }
    }
}
