use crate::ctx::Context;
use crate::dbs::{Options, Transaction};
use crate::doc::CursorDoc;
use crate::err::Error;
use crate::sql::comment::shouldbespace;
use crate::sql::error::IResult;
use crate::sql::number::Number;
use crate::sql::value::{value, Value};
use nom::bytes::complete::tag_no_case;
use nom::combinator::{cut, opt};
use nom::sequence::terminated;
use revision::revisioned;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Serialize, Deserialize, Hash)]
#[revisioned(revision = 1)]
pub struct Start(pub Value);

impl Start {
	pub(crate) async fn process(
		&self,
		ctx: &Context<'_>,
		opt: &Options,
		txn: &Transaction,
		doc: Option<&CursorDoc<'_>>,
	) -> Result<usize, Error> {
		match self.0.compute(ctx, opt, txn, doc).await {
			// This is a valid starting number
			Ok(Value::Number(Number::Int(v))) if v >= 0 => Ok(v as usize),
			// An invalid value was specified
			Ok(v) => Err(Error::InvalidStart {
				value: v.as_string(),
			}),
			// A different error occured
			Err(e) => Err(e),
		}
	}
}

impl fmt::Display for Start {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "START {}", self.0)
	}
}

pub fn start(i: &str) -> IResult<&str, Start> {
	let (i, _) = tag_no_case("START")(i)?;
	let (i, _) = shouldbespace(i)?;
	cut(|i| {
		let (i, _) = opt(terminated(tag_no_case("AT"), shouldbespace))(i)?;
		let (i, v) = value(i)?;
		Ok((i, Start(v)))
	})(i)
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn start_statement() {
		let sql = "START 100";
		let res = start(sql);
		let out = res.unwrap().1;
		assert_eq!(out, Start(Value::from(100)));
		assert_eq!("START 100", format!("{}", out));
	}

	#[test]
	fn start_statement_at() {
		let sql = "START AT 100";
		let res = start(sql);
		let out = res.unwrap().1;
		assert_eq!(out, Start(Value::from(100)));
		assert_eq!("START 100", format!("{}", out));
	}
}
