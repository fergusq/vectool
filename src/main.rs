mod lexer;
mod model;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::binary_heap::BinaryHeap;
use std::cmp::Ordering;
#[macro_use] extern crate lazy_static;
use regex::Regex;
use rulinalg::vector::Vector;
use rulinalg::norm::Euclidean;
use clap::{App, AppSettings, Arg, SubCommand};

use model::Model;
use lexer::Token;

fn load_excluded_file(filename: &str) -> impl Iterator<Item = String> {
	let reader = BufReader::new(File::open(filename).expect("exclude file could not be opened"));
	reader.lines().flatten().map(preprocess_text)
}

fn preprocess_text(text: String) -> String {
	lazy_static!{
		static ref special_character_regex: Regex = Regex::new("[^a-zåäö ]").unwrap();
	}
	special_character_regex.replace_all(text.to_lowercase().as_str(), "").to_string()
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

fn filter(model: &Model) {
	let stdin = std::io::stdin();
	for line in stdin.lock().lines() {
		for token in lexer::lex(line.unwrap()) {
			match token {
				Token::Other(s) => print!("{}", s),
				Token::Word(w, c) => print!("{}", lexer::capitalize(find_nn(model, w), c))
			}
		}
		println!();
	}
}

fn nn(model: &Model) {
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
		for (distance, near_word) in find_nnk(model, &vector, 10) {
			if words.contains(&near_word.as_str()) {
				println!("({} {:.4})", near_word, distance);
			} else {
				println!("{} {:.4}", near_word, distance);
			}
		}
	}
}

fn main() {
	let matches = App::new("Vectool")
		.version("0.1.0")
		.author("Iikka Hauhio <iikka.hauhio@helsinki.fi>")
		.about("Make queries to word vector models")
		.arg(Arg::with_name("model")
			.help("The word vector model file")
			.required(true))
		.arg(Arg::with_name("exclude_file")
			.help("Set the file of excluded words")
			.short("-e")
			.long("exclude-file")
			.takes_value(true))
		.subcommand(SubCommand::with_name("filter")
			.about("Replace words in the input stream with their nearest neighbours"))
		.subcommand(SubCommand::with_name("nn")
			.about("Find nearest neighbours of linear combinations of word vectors"))
		.setting(AppSettings::SubcommandRequired)
		.get_matches();
	
	let model_file = matches.value_of("model").unwrap();
	let mut model = model::load_model(model_file);
	eprintln!("Loaded {} word model", model.0.len());

	if let Some(exclude_file) = matches.value_of("exclude_file") {
		let excluded = load_excluded_file(exclude_file);
		for word in excluded {
			model.0.remove(&word);
		}
	}

	if let Some(_subcommand_matches) = matches.subcommand_matches("filter") {
		filter(&model);
	} else if let Some(_subcommand_matches) = matches.subcommand_matches("nn") {
		nn(&model);
	}
}
