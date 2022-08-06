use std::collections::VecDeque;
use std::convert::TryFrom;
use std::fmt::Display;

// TODO Add 'em comments.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Operator {
	Add = 1,
	Sub = 7,
	Mul = 2,
	Div = 8,
}

impl Display for Operator {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::Add => "+",
			Self::Sub => "-",
			Self::Mul => "*",
			Self::Div => "/",
		};

		text.fmt(f)
	}
}

impl Operator {
	fn precedes(self, rhs: Self) -> bool {
		((self as u8) % 6) >= ((rhs as u8) % 6) // FIXME Rid of magic numbers
	}

	fn perform(&self, a: i32, b: i32) -> i32 {
		match self {
			Operator::Add => a + b,
			Operator::Sub => a - b,
			Operator::Mul => a * b,
			Operator::Div => a / b,
		}
	}
}

#[derive(Debug)]
pub enum Paren {
	Left,
	Right,
}

impl Display for Paren {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let c = match self {
			Self::Left => "(",
			Self::Right => ")",
		};

		c.fmt(f)
	}
}

#[derive(Debug)]
pub enum InToken {
	Num(i32),
	Op(Operator),
	Paren(Paren),
}

#[derive(Debug)]
pub enum OpStackToken {
	Op(Operator),
	LeftParen,
}

#[derive(Debug)]
pub enum OutToken {
	Num(i32),
	Op(Operator),
}

impl OpStackToken {
	fn precedes(&self, rhs: Operator) -> bool {
		match self {
			Self::LeftParen => false,
			Self::Op(op) => op.precedes(rhs),
		}
	}
}

impl TryFrom<OpStackToken> for OutToken {
	type Error = UnbalancedParen;

	fn try_from(token: OpStackToken) -> Result<Self, Self::Error> {
		match token {
			OpStackToken::Op(op) => Ok(Self::Op(op)),
			OpStackToken::LeftParen => Err(UnbalancedParen(Paren::Left)),
		}
	}
}

impl From<OutToken> for String {
	fn from(token: OutToken) -> Self {
		match token {
			OutToken::Num(n) => n.to_string(),
			OutToken::Op(op) => op.to_string(),
		}
	}
}

impl Display for InToken {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Num(n) => n.fmt(f),
			Self::Op(op) => op.fmt(f),
			Self::Paren(paren) => paren.fmt(f),
		}
	}
}

#[derive(Debug)]
pub enum ResolveError {
	InvalidToken(char),
	UnbalancedParen(Paren),
	NotEnoughOperands,
	NoValue,
}

#[derive(Debug)]
pub struct InvalidToken(char);

#[derive(Debug)]
pub struct UnbalancedParen(Paren);

#[derive(Debug)]
pub struct NotEnoughOperands;

impl From<NotEnoughOperands> for ResolveError {
	fn from(_: NotEnoughOperands) -> Self {
		Self::NotEnoughOperands
	}
}

impl From<UnbalancedParen> for ResolveError {
	fn from(unbalanced_paren: UnbalancedParen) -> Self {
		Self::UnbalancedParen(unbalanced_paren.0)
	}
}

impl From<InvalidToken> for ResolveError {
	fn from(invalid_token: InvalidToken) -> Self {
		Self::InvalidToken(invalid_token.0)
	}
}

impl From<Operator> for InToken {
	fn from(op: Operator) -> Self {
		Self::Op(op)
	}
}

impl From<Operator> for OpStackToken {
	fn from(op: Operator) -> Self {
		Self::Op(op)
	}
}

impl From<Paren> for InToken {
	fn from(paren: Paren) -> Self {
		Self::Paren(paren)
	}
}

impl TryFrom<char> for InToken {
	type Error = InvalidToken;

	fn try_from(string: char) -> Result<Self, Self::Error> {
		Ok(match string {
			'+' => Operator::Add.into(),
			'-' => Operator::Sub.into(),
			'*' => Operator::Mul.into(),
			'/' => Operator::Div.into(),
			'(' => Paren::Left.into(),
			')' => Paren::Right.into(),
			c => c
				.to_digit(10)
				.map(|digit| Self::Num(digit as i32))
				.ok_or(InvalidToken(c))?,
		})
	}
}

fn pop_until_left_paren(
	output: &mut Vec<OutToken>,
	ops: &mut Vec<OpStackToken>,
) -> Result<(), UnbalancedParen> {
	while let Some(op) = ops.pop() {
		if let OpStackToken::LeftParen = op {
			return Ok(());
		}

		output.push(op.try_into()?);
	}

	Err(UnbalancedParen(Paren::Right))
}

fn handle_operation_parsing(op: Operator, output: &mut Vec<OutToken>, ops: &mut Vec<OpStackToken>) {
	// While the operator at the top of the operator stack has a higher precedence than `op`,
	// pop it off and push it to the output queue.
	while ops.last().map(|top| top.precedes(op)).unwrap_or(false) {
		let op = ops.pop().and_then(|op| op.try_into().ok()).unwrap();
		output.push(op);
	}

	ops.push(op.into())
}

fn handle_operation_evaluation(
	op: Operator,
	numbers: &mut VecDeque<i32>,
) -> Result<(), NotEnoughOperands> {
	let result = numbers
		.pop_front()
		.zip(numbers.pop_front())
		.map(|(rop, lop)| op.perform(lop, rop))
		.ok_or(NotEnoughOperands)?;

	numbers.push_front(result);

	Ok(())
}

// TODO Make this work for multi digit numbers.
fn parse_into_tokens(expr: &str) -> Result<Vec<OutToken>, ResolveError> {
	let mut output = Vec::new();
	let mut ops = Vec::new();

	let tokens = expr
		.chars()
		.map(InToken::try_from)
		.collect::<Result<Vec<InToken>, _>>()?;

	for token in tokens.into_iter() {
		match token {
			InToken::Num(n) => output.push(OutToken::Num(n)),
			InToken::Op(op) => handle_operation_parsing(op, &mut output, &mut ops),
			InToken::Paren(paren) => match paren {
				Paren::Left => ops.push(OpStackToken::LeftParen),
				Paren::Right => pop_until_left_paren(&mut output, &mut ops)?,
			},
		}
	}

	while let Some(op) = ops.pop() {
		output.push(op.try_into()?)
	}

	Ok(output)
}

pub fn parse(expr: &str) -> Result<String, ResolveError> {
	Ok(parse_into_tokens(expr)?
		.into_iter()
		.map(String::from)
		.collect())
}

pub fn eval(expr: &str) -> Result<i32, ResolveError> {
	let tokens = parse_into_tokens(expr)?;
	let mut numbers: VecDeque<i32> = VecDeque::new();

	for token in tokens.into_iter() {
		match token {
			OutToken::Num(n) => numbers.push_front(n),
			OutToken::Op(op) => handle_operation_evaluation(op, &mut numbers)?,
		}
	}

	numbers.pop_front().ok_or(ResolveError::NoValue)
}

// 1+2-(2+1)*2
//
// 12+21+2*-
// ______________________           __________________
//       output                         input
//
//  op_stack:
//
//
//
//
//
//
//
// ______________________           __________________
//       input                          output
//
//  working_stack:
//  num_queue: 36
//

#[cfg(test)]
mod tests {
	use super::*;
	use Operator::*;

	#[test]
	fn add_precedes_sub() {
		assert!(Add.precedes(Sub))
	}

	#[test]
	fn sub_precedes_add() {
		assert!(Sub.precedes(Add))
	}

	#[test]
	fn mul_precedes_div() {
		assert!(Mul.precedes(Div))
	}

	#[test]
	fn div_precedes_mul() {
		assert!(Div.precedes(Mul))
	}

	#[test]
	fn mul_precedes_add() {
		assert!(Mul.precedes(Add))
	}

	#[test]
	fn div_precedes_add() {
		assert!(Div.precedes(Add))
	}

	#[test]
	fn mul_precedes_sub() {
		assert!(Mul.precedes(Sub))
	}

	#[test]
	fn add_does_not_precede_mul() {
		assert!(!Add.precedes(Mul))
	}

	#[test]
	fn sub_does_not_precede_mul() {
		assert!(!Sub.precedes(Mul))
	}

	#[test]
	fn sub_does_not_precede_div() {
		assert!(!Sub.precedes(Div))
	}

	#[test]
	fn add_does_not_precede_div() {
		assert!(!Add.precedes(Div))
	}

	#[test]
	fn parsing_works() {
		assert!(matches!(parse("1+s"), Err(ResolveError::InvalidToken('s'))));
		assert!(matches!(
			parse("1+2-8)"),
			Err(ResolveError::UnbalancedParen(Paren::Right))
		));
		assert!(matches!(
			parse("(1+2-8"),
			Err(ResolveError::UnbalancedParen(Paren::Left))
		));
		assert!(matches!(
			parse(")))"),
			Err(ResolveError::UnbalancedParen(Paren::Right))
		));
		assert_eq!(parse("1+2-(2+1)*2").unwrap(), "12+21+2*-");
		assert_eq!(parse("2+(3*(8-4))").unwrap(), "2384-*+");
		assert_eq!(parse("(0)").unwrap(), "0");
		assert_eq!(parse("").unwrap(), "");
		assert_eq!(parse("(())").unwrap(), "");
	}

	#[test]
	fn eval_works() {
		assert_eq!(eval("1+2-(2+1)*2").unwrap(), -3);
		assert_eq!(eval("2+(3*(8-4))").unwrap(), 14);
		assert_eq!(eval("0").unwrap(), 0);
		assert_eq!(eval("(0)").unwrap(), 0);
		assert_eq!(eval("(((0-1)))").unwrap(), -1);
		assert!(matches!(eval("expr"), Err(ResolveError::InvalidToken('e'))));
		assert!(matches!(
			eval("))"),
			Err(ResolveError::UnbalancedParen(Paren::Right))
		));
		assert!(matches!(eval("(())"), Err(ResolveError::NoValue)));
		assert!(matches!(eval(""), Err(ResolveError::NoValue)));
		assert!(matches!(
			eval("("),
			Err(ResolveError::UnbalancedParen(Paren::Left))
		));
	}
}
