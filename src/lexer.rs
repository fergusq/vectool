#[derive(Debug, PartialEq, Eq)]
pub enum Token {
	Word(String, Capitalization),
	Other(String)
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Capitalization {
	LowerCase,
	UpperCase,
	Capitalized
}

fn determine_capitalization(word: String) -> Capitalization {
	if word.is_empty() {
		Capitalization::LowerCase // doesn't matter
	} else if word == word.to_lowercase() {
		Capitalization::LowerCase
	} else if word == word.to_uppercase() {
		Capitalization::UpperCase
	} else {
		Capitalization::Capitalized
	}
}

pub fn capitalize(word: String, capitalization: Capitalization) -> String {
	match capitalization {
		Capitalization::LowerCase => word.to_lowercase(),
		Capitalization::UpperCase => word.to_uppercase(),
		Capitalization::Capitalized => {
			let mut cs = word.chars();
			match cs.next() {
				None => word,
				Some(c) => c.to_uppercase().to_string() + cs.as_str()
			}
		}
	}
}

pub fn lex(text: String) -> Vec<Token> {
	let mut ans = Vec::new();
	let mut word = String::new();
	let mut other = String::new();
	for chr in text.trim().chars() {
		match chr {
			'A'...'Z' | 'Ä' | 'Ö' | 'Å' | 'a'...'z' | 'ä' | 'ö' | 'å' => {
				if !other.is_empty() {
					if word.is_empty() {
						ans.push(Token::Other(other));
					}
					other = String::new();
				}
				word.push(chr);
			}
			' ' => {
				if !word.is_empty() {
					ans.push(Token::Word(word.to_lowercase(), determine_capitalization(word)));
					word = String::new();
				}
				if !other.is_empty() {
					ans.push(Token::Other(other));
					other = String::new();
				}
				ans.push(Token::Other(String::from(" ")));
			}
			_ => {
				other.push(chr);
			}
		}
	}
	if !word.is_empty() {
		ans.push(Token::Word(word.to_lowercase(), determine_capitalization(word)));
	}
	if !other.is_empty() {
		ans.push(Token::Other(other));
	}
	ans
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_capitalize() {
		assert_eq!(capitalize(String::from("kissa"), Capitalization::UpperCase), "KISSA");
		assert_eq!(capitalize(String::from("talo"), Capitalization::LowerCase), "talo");
		assert_eq!(capitalize(String::from("äiti"), Capitalization::Capitalized), "Äiti");
	}

	#[test]
	fn test_lex() {
		assert_eq!(
			lex(String::from("D-N: Världens vackraste katter lever i Sverige!")),
			vec![
				Token::Word(String::from("dn"), Capitalization::UpperCase),
				Token::Other(String::from(":")),
				Token::Other(String::from(" ")),
				Token::Word(String::from("världens"), Capitalization::Capitalized),
				Token::Other(String::from(" ")),
				Token::Word(String::from("vackraste"), Capitalization::LowerCase),
				Token::Other(String::from(" ")),
				Token::Word(String::from("katter"), Capitalization::LowerCase),
				Token::Other(String::from(" ")),
				Token::Word(String::from("lever"), Capitalization::LowerCase),
				Token::Other(String::from(" ")),
				Token::Word(String::from("i"), Capitalization::LowerCase),
				Token::Other(String::from(" ")),
				Token::Word(String::from("sverige"), Capitalization::Capitalized),
				Token::Other(String::from("!"))
			]
		);
	}
}