mod calc;
mod lexer;
mod model;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::{binary_heap::BinaryHeap, HashMap};
use std::cmp::Ordering;
#[macro_use] extern crate lazy_static;
use regex::Regex;
use rulinalg::vector::Vector;
use rulinalg::norm::Euclidean;
use clap::{App, AppSettings, Arg, SubCommand};
#[macro_use] extern crate nom;

use calc::Expression;
use model::Model;
use lexer::Token;

fn load_excluded_file(filename: &str) -> impl Iterator<Item = String> {
	let reader = BufReader::new(File::open(filename).expect("exclude file could not be opened"));
	reader.lines().flatten().map(|s| preprocess_text(&s))
}

fn preprocess_text(text: &str) -> String {
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

fn calc(model: &Model) {
	let stdin = std::io::stdin();
	'outer: for line in stdin.lock().lines() {
		let mut preprocessed_line = line.unwrap()
			.trim().to_string();
		preprocessed_line.push('.');
		
		let expr = calc::expr(preprocessed_line.as_str());
		match expr {
			Ok((".", x)) => {
				let words = x.words();
				for word in &words {
					if !model.0.contains_key(word) {
						eprintln!("unknown word `{}'", word);
						continue 'outer;
					}
				}
				match x {
					Expression::NN(e) => {
						let vector = calc::eval(e, model).unwrap();
						for (distance, near_word) in find_nnk(model, &vector, 10) {
							if words.contains(&near_word) {
								println!("({} {:.4})", near_word, distance);
							} else {
								println!("{} {:.4}", near_word, distance);
							}
						}
					},
					Expression::Distance(a, b) => {
						let x = calc::eval(a, model).unwrap();
						let y = calc::eval(b, model).unwrap();
						println!("Cosine distance: {}", x.dot(&y) / x.norm(Euclidean) / y.norm(Euclidean));
						println!("Euclidean distance: {}", (x - y).norm(Euclidean));
					}
				}
			}
			_ => eprintln!("syntax error"),
		}
	}
}

fn sa(model: &Model, subs: HashMap<String, String>) {
	let stdin = std::io::stdin();
	for line in stdin.lock().lines() {
		let line = preprocess_text(line.unwrap().trim());
		let words = line.split(|c: char| c.is_whitespace());
		let mut result = Vec::new();
		'middle: for word in words {
			if subs.contains_key(word) {
				result.push(subs[word].clone());
				continue;
			}
			else if model.0.contains_key(word) {
				let vw = &model.0[word];
				for (a, b) in &subs {
					if model.0.contains_key(a) && model.0.contains_key(b) {
						let va = &model.0[a];
						let vb = &model.0[b];
						let analogy = vw - va + vb;
						for (distance, near_word) in find_nnk(model, &analogy, 4) {
							if near_word != word && near_word != *a && near_word != *b && distance >= 0.6 {
								result.push(near_word);
								continue 'middle;
							}
						}
					}
				}
			}
			result.push(word.to_string());
		}
		println!("{}", result.join(" "));
	}
}

fn main() {
	let matches = App::new("Vectool")
		.version("0.3.0")
		.author("Iikka Hauhio <iikka.hauhio@helsinki.fi>")
		.about("Make queries to word vector models")
		.arg(Arg::with_name("model")
			.help("The word vector model file")
			.required(true)
		)
		.arg(Arg::with_name("exclude_file")
			.help("Set the file of excluded words")
			.short("-e")
			.long("exclude-file")
			.takes_value(true)
		)
		.subcommand(SubCommand::with_name("filter")
			.about("Replace words in the input stream with their nearest neighbours")
		)
		.subcommand(SubCommand::with_name("calc")
			.about("Find nearest neighbours of linear combinations of word vectors and compare vectors")
		)
		.subcommand(SubCommand::with_name("sa")
			.about("Create sentence analogies")
			.arg(Arg::with_name("substitutions")
				.help("Substitution list")
				.multiple(true)
				.value_name("FROM=TO")
				.validator(|s| if s.contains('=') {Ok(())} else {Err("Substitutions must have form FROM=TO".to_string())})
			)
		)
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
	} else if let Some(_subcommand_matches) = matches.subcommand_matches("calc") {
		calc(&model);
	} else if let Some(subcommand_matches) = matches.subcommand_matches("sa") {
		let subs = subcommand_matches.values_of("substitutions").unwrap()
			.map(|v| v.split('=').map(String::from).collect::<Vec<_>>())
			.map(|a| (a[0].clone(), a[1].clone()))
			.collect::<HashMap<_, _>>();
		sa(&model, subs);
	}
}
