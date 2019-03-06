use rulinalg::vector::Vector;
use crate::model::Model;

// Parser

#[derive(Debug, Clone, PartialEq)]
pub enum VecExpression {
	Word(String),
	Add(Box<VecExpression>, Box<VecExpression>),
	Sub(Box<VecExpression>, Box<VecExpression>)
}

impl VecExpression {
	pub fn words(&self) -> Vec<String> {
		match self {
			VecExpression::Word(w) => vec![w.clone()],
			VecExpression::Add(a, b) => [a.words(), b.words()].concat(),
			VecExpression::Sub(a, b) => [a.words(), b.words()].concat()
		}
	}
}

fn add_expr(a: VecExpression, b: VecExpression) -> VecExpression {
	VecExpression::Add(Box::new(a), Box::new(b))
}

fn sub_expr(a: VecExpression, b: VecExpression) -> VecExpression {
	VecExpression::Sub(Box::new(a), Box::new(b))
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
	NN(VecExpression),
	Distance(VecExpression, VecExpression)
}

impl Expression {
	pub fn words(&self) -> Vec<String> {
		match self {
			Expression::NN(e) => e.words(),
			Expression::Distance(a, b) => [a.words(), b.words()].concat()
		}
	}
}

fn is_word_letter(c: char) -> bool {
	match c {
		'a'...'z' | 'ä' | 'ö' | 'å' => true,
		_ => false
	}
}

named!(word<&str, VecExpression>,
	map!(ws!(take_while!(is_word_letter)), |s| VecExpression::Word(s.to_string()))
);

named!(vec_expr<&str, VecExpression>, do_parse!(
	init: word >>
	terms: fold_many0!(
		tuple!(alt!(tag!("+") | tag!("-")), word),
		init,
		|acc, o: (_, VecExpression)| if o.0 == "+" {add_expr(acc, o.1)} else {sub_expr(acc, o.1)}
	) >>
	(terms)
));

named!(pub expr<&str, Expression>, do_parse!(
	init: vec_expr >>
	dis: opt!(tuple!(tag!("<>"), vec_expr)) >>
	(if let Some(v) = dis {Expression::Distance(init, v.1)} else {Expression::NN(init)})
));

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_word() {
		assert_eq!(word("kissa."), Ok((".", VecExpression::Word("kissa".to_string()))));
	}

	#[test]
	fn test_vecexpr() {
		assert_eq!(vec_expr("kissa."), Ok((".", VecExpression::Word("kissa".to_string()))));
		assert_eq!(vec_expr("kissa + koira."), Ok((
			".", VecExpression::Add(
				Box::new(VecExpression::Word("kissa".to_string())),
				Box::new(VecExpression::Word("koira".to_string()))
			)
		)));
	}

	#[test]
	fn test_expr() {
		assert_eq!(expr("kissa + koira."), Ok((
			".", Expression::NN(VecExpression::Add(
				Box::new(VecExpression::Word("kissa".to_string())),
				Box::new(VecExpression::Word("koira".to_string()))
			))
		)));
		assert_eq!(expr("kissa + koira <> kala - hevonen."), Ok((
			".", Expression::Distance(VecExpression::Add(
				Box::new(VecExpression::Word("kissa".to_string())),
				Box::new(VecExpression::Word("koira".to_string()))
			), VecExpression::Sub(
				Box::new(VecExpression::Word("kala".to_string())),
				Box::new(VecExpression::Word("hevonen".to_string()))
			))
		)));
	}
}

// Interpreter

pub fn eval(expr: VecExpression, model: &Model) -> Option<Vector<f64>> {
	match expr {
		VecExpression::Word(word) => model.0.get(word.as_str()).map(|v| v.clone()),
		VecExpression::Add(a, b) => match (eval(*a, model), eval(*b, model)) {
			(Some(x), Some(y)) => Some(x + y),
			_ => None
		}
		VecExpression::Sub(a, b) => match (eval(*a, model), eval(*b, model)) {
			(Some(x), Some(y)) => Some(x - y),
			_ => None
		}
	}
}