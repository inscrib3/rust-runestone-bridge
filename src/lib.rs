#![deny(clippy::all)]

use std::{io::Cursor, str::FromStr};

use bitcoin::{consensus::Decodable, Transaction};
use napi::bindgen_prelude::{BigInt, Buffer};
use ordinals::{Artifact, SpacedRune};

#[macro_use]
extern crate napi_derive;

#[napi(object)]
#[derive(Clone)]
pub struct Edict {
  pub id: String,
  pub amount: BigInt,
  pub output: u32,
}

impl From<ordinals::Edict> for Edict {
  fn from(value: ordinals::Edict) -> Self {
    Edict {
      id: format!("{}", value.id),
      amount: value.amount.into(),
      output: value.output,
    }
  }
}

impl TryFrom<Edict> for ordinals::Edict {
  type Error = Box<dyn std::error::Error>;

  fn try_from(value: Edict) -> Result<Self, Self::Error> {
    let id: ordinals::RuneId = value.id.parse()?;
    Ok(Self {
      id,
      amount: value.amount.get_u128().1,
      output: value.output,
    })
  }
}

#[napi(object)]
#[derive(Clone)]
pub struct BlockRange {
  pub start: Option<BigInt>,
  pub end: Option<BigInt>,
}

impl From<(Option<u64>, Option<u64>)> for BlockRange {
  fn from(value: (Option<u64>, Option<u64>)) -> Self {
    BlockRange {
      start: value.0.map(|v| v.into()),
      end: value.1.map(|v| v.into()),
    }
  }
}

impl From<BlockRange> for (Option<u64>, Option<u64>) {
  fn from(value: BlockRange) -> Self {
    (
      value.start.map(|v| v.get_u64().1),
      value.end.map(|v| v.get_u64().1),
    )
  }
}

#[napi(object)]
#[derive(Clone)]
pub struct Terms {
  pub amount: Option<BigInt>,
  pub cap: Option<BigInt>,
  pub height: BlockRange,
  pub offset: BlockRange,
}

impl From<ordinals::Terms> for Terms {
  fn from(value: ordinals::Terms) -> Self {
    Terms {
      amount: value.amount.map(|v| v.into()),
      cap: value.cap.map(|v| v.into()),
      height: value.height.into(),
      offset: value.offset.into(),
    }
  }
}

impl From<Terms> for ordinals::Terms {
  fn from(value: Terms) -> Self {
    Self {
      amount: value.amount.map(|v| v.get_u128().1),
      cap: value.cap.map(|v| v.get_u128().1),
      height: value.height.into(),
      offset: value.offset.into(),
    }
  }
}

#[napi(object)]
#[derive(Clone)]
pub struct Etching {
  pub divisibility: Option<u8>,
  pub premine: Option<BigInt>,
  pub rune: String,
  pub symbol: String,
  pub terms: Option<Terms>,
  pub turbo: bool,
}

impl From<ordinals::Etching> for Etching {
  fn from(value: ordinals::Etching) -> Self {
    Self {
      divisibility: value.divisibility.map(|v| v.into()),
      premine: value.premine.map(|v| v.into()),
      rune: value
        .rune
        .map(|v| SpacedRune::new(v, value.spacers.unwrap_or_default()).to_string())
        .unwrap_or_default(),
      symbol: value.symbol.map(|v| v.to_string()).unwrap_or_default(),
      terms: value.terms.map(|v| v.into()),
      turbo: value.turbo,
    }
  }
}

impl TryFrom<Etching> for ordinals::Etching {
  type Error = Box<dyn std::error::Error>;

  fn try_from(value: Etching) -> Result<Self, Self::Error> {
    let rune = SpacedRune::from_str(&value.rune)?;
    Ok(Self {
      divisibility: value.divisibility,
      premine: value.premine.map(|v| v.get_u128().1),
      rune: Some(rune.rune),
      spacers: Some(rune.spacers),
      symbol: value.symbol.chars().next(),
      terms: value.terms.map(|v| v.into()),
      turbo: value.turbo,
    })
  }
}

#[napi(object)]
#[derive(Clone)]
pub struct Runestone {
  pub edicts: Vec<Edict>,
  pub etching: Option<Etching>,
  pub mint: Option<String>,
  pub pointer: Option<u32>,
}

impl From<ordinals::Runestone> for Runestone {
  fn from(value: ordinals::Runestone) -> Self {
    Self {
      edicts: value.edicts.into_iter().map(|v| v.into()).collect(),
      etching: value.etching.map(|v| v.into()),
      mint: value.mint.map(|v| v.to_string()),
      pointer: value.pointer,
    }
  }
}

impl TryFrom<Runestone> for ordinals::Runestone {
  type Error = Box<dyn std::error::Error>;

  fn try_from(value: Runestone) -> Result<Self, Self::Error> {
    let rune_id = value
      .mint
      .map(|v| ordinals::RuneId::from_str(&v))
      .transpose()?;
    Ok(Self {
      edicts: value
        .edicts
        .into_iter()
        .map(|v| ordinals::Edict::try_from(v))
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?,
      etching: value.etching.map(|v| v.try_into()).transpose()?,
      mint: rune_id,
      pointer: value.pointer,
    })
  }
}

#[napi]
pub fn decipher(transaction: String) -> Option<Runestone> {
  let hex = hex::decode(transaction).ok()?;
  let mut cursor = Cursor::new(hex);
  let transaction = Transaction::consensus_decode(&mut cursor).ok()?;

  let runestone = ordinals::Runestone::decipher(&transaction)
    .ok_or_else(|| String::from("No artifact found for transaction"))
    .ok()?;
  match runestone {
    Artifact::Cenotaph(_) => None,
    Artifact::Runestone(runestone) => {
      return Some(runestone.into());
    }
  }
}

#[napi]
pub fn encipher(data: Runestone) -> Option<Buffer> {
  let runestone: ordinals::Runestone = data.try_into().ok()?;
  let res: Vec<u8> = runestone.encipher().into();
  Some(res.into())
}

