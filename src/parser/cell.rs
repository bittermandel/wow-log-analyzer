use std::str::{from_utf8, FromStr};

use dioxus::prelude::SvgAttributes;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::{
        complete::{alpha1, alphanumeric1, char, digit1, hex_digit1, not_line_ending, one_of},
        is_alphanumeric, is_digit, is_newline,
    },
    combinator::{self, all_consuming, map, map_res, opt, recognize},
    error::ErrorKind,
    multi::{many0, many1, separated_list0, separated_list1},
    number::{
        self,
        complete::{double, float},
    },
    sequence::{delimited, pair, separated_pair, terminated, tuple},
    Err, IResult, Parser,
};

use crate::parser;

#[derive(Debug, PartialEq, Clone)]
pub enum LogCell<'a> {
    Integer(i64),
    Float(f64),
    MultiPowerCell((i64, i64)),
    Str(&'a str),
    Array(Vec<LogCell<'a>>),
}

impl<'a> From<LogCell<'a>> for bool {
    fn from(cell: LogCell) -> Self {
        match cell {
            LogCell::Integer(v) => v != 0,
            LogCell::Float(v) => v != 0.0,
            LogCell::MultiPowerCell(v) => v.0 != 0,
            LogCell::Str(v) => v != "",
            LogCell::Array(v) => v.len() != 0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LogRow<'a> {
    Emote(LogEmote<'a>),
    SpellCastSuccess(LogSpellCastSuccess<'a>),
    SpellDamage(LogSpellDamage<'a>),
    SpellHeal(LogSpellHeal<'a>),
    NotSupported,
}

#[derive(Debug, PartialEq)]
pub struct LogEmote<'a> {
    pub sourceGUID: &'a str,
    pub sourcename: &'a str,
    pub sourceflags: &'a str,
    pub sourceraidflags: &'a str,
    pub text: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct LogSpellCastSuccess<'a> {
    sourceGUID: LogCell<'a>,
    sourceName: LogCell<'a>,
    sourceFlags: LogCell<'a>,
    sourceRaidFlags: LogCell<'a>,
    destGUID: LogCell<'a>,
    destName: LogCell<'a>,
    destFlags: LogCell<'a>,
    destRaidFlags: LogCell<'a>,
    spellId: LogCell<'a>,
    spellName: LogCell<'a>,
    spellSchool: LogCell<'a>,
    unitGUID: LogCell<'a>,
    ownerGUID: LogCell<'a>,
    currHp: LogCell<'a>,
    maxHp: LogCell<'a>,
    attackPower: LogCell<'a>,
    spellPower: LogCell<'a>,
    armor: LogCell<'a>,
    totalDamageAbsorbs: LogCell<'a>,
    resourceType: LogCell<'a>,
    currResource: LogCell<'a>,
    maxResource: LogCell<'a>,
    resourceCost: LogCell<'a>,
    y: LogCell<'a>,
    x: LogCell<'a>,
    mapId: LogCell<'a>,
    facing: LogCell<'a>,
    ilvl: LogCell<'a>,
}

#[derive(Debug, PartialEq)]
pub struct LogSpellDamage<'a> {
    sourceGUID: LogCell<'a>,
    sourceName: LogCell<'a>,
    sourceFlags: LogCell<'a>,
    sourceRaidFlags: LogCell<'a>,
    destGUID: LogCell<'a>,
    destName: LogCell<'a>,
    destFlags: LogCell<'a>,
    destRaidFlags: LogCell<'a>,
    spellId: LogCell<'a>,
    spellName: LogCell<'a>,
    spellSchool: LogCell<'a>,
    unitGUID: LogCell<'a>,
    ownerGUID: LogCell<'a>,
    currHp: LogCell<'a>,
    maxHp: LogCell<'a>,
    attackPower: LogCell<'a>,
    spellPower: LogCell<'a>,
    armor: LogCell<'a>,
    totalDamageAbsorbs: LogCell<'a>,
    resourceType: LogCell<'a>,
    currResource: LogCell<'a>,
    maxResource: LogCell<'a>,
    resourceCost: LogCell<'a>,
    y: LogCell<'a>,
    x: LogCell<'a>,
    mapId: LogCell<'a>,
    facing: LogCell<'a>,
    ilvl: LogCell<'a>,
    amount: LogCell<'a>,
    overkill: LogCell<'a>,
    school: LogCell<'a>,
    resisted: LogCell<'a>,
    blocked: LogCell<'a>,
    absorbed: LogCell<'a>,
    critical: bool,
    glancing: bool,
    crushing: bool,
    isOffHand: bool,
}

#[derive(Debug, PartialEq)]
pub struct LogSpellHeal<'a> {
    sourceGUID: LogCell<'a>,
    sourceName: LogCell<'a>,
    sourceFlags: LogCell<'a>,
    sourceRaidFlags: LogCell<'a>,
    destGUID: LogCell<'a>,
    destName: LogCell<'a>,
    destFlags: LogCell<'a>,
    destRaidFlags: LogCell<'a>,
    spellId: LogCell<'a>,
    spellName: LogCell<'a>,
    spellSchool: LogCell<'a>,
    unitGUID: LogCell<'a>,
    ownerGUID: LogCell<'a>,
    currHp: LogCell<'a>,
    maxHp: LogCell<'a>,
    attackPower: LogCell<'a>,
    spellPower: LogCell<'a>,
    armor: LogCell<'a>,
    totalDamageAbsorbs: LogCell<'a>,
    resourceType: LogCell<'a>,
    currResource: LogCell<'a>,
    maxResource: LogCell<'a>,
    resourceCost: LogCell<'a>,
    y: LogCell<'a>,
    x: LogCell<'a>,
    mapId: LogCell<'a>,
    facing: LogCell<'a>,
    ilvl: LogCell<'a>,
    amount: LogCell<'a>,
    overhealing: LogCell<'a>,
    absorbed: LogCell<'a>,
    critical: bool,
}

#[derive(Debug, PartialEq)]
pub struct LogEventDateTime<'a> {
    // The month an event occurred
    pub month: &'a str,
    // The day of the month an event occurred
    pub day: &'a str,
    // The hour an event occured
    pub hour: &'a str,
    // The minute an event occured
    pub minute: &'a str,
    // The second event occured
    pub second: &'a str,
    // The millisecond event occured
    pub ms: &'a str,
}

pub fn parse_log_cell(input: &str) -> IResult<&str, LogCell> {
    if input.len() == 1 {
        return match &input[0..1] {
            "[" => parse_array(input, "[".to_string(), "]".to_string()),
            "(" => parse_array(input, "(".to_string(), ")".to_string()),
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "-" => parse_integer(input),
            _ => parse_string(input),
        };
    }
    match &input[0..1] {
        "[" => parse_array(input, "[".to_string(), "]".to_string()),
        "(" => parse_array(input, "(".to_string(), ")".to_string()),
        "0" => match &input[0..2] {
            "0x" => {
                let parser = tuple((tag("0x"), alphanumeric1));
                map(parser, |(_, v)| LogCell::Str(v))(input)
            }
            _ => parse_number(input),
        },
        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "-" => parse_number(input),
        _ => parse_string(input),
    }
}

pub fn parse_array(
    input: &str,
    start_delimiter: String,
    end_delimiter: String,
) -> IResult<&str, LogCell> {
    let parser = delimited(
        tag(start_delimiter.as_str()),
        separated_list0(tag(","), parse_log_cell),
        tag(end_delimiter.as_str()),
    );

    map(parser, |v| LogCell::Array(v))(input)
}

pub fn parse_string(input: &str) -> IResult<&str, LogCell> {
    match &input[0..1] {
        "|" => {
            let parser = delimited(tag("|T"), take_while1(is_valid_emote), tag("!"));

            map(parser, |v| LogCell::Str(v))(input)
        }
        "\"" => {
            let parser = delimited(tag("\""), take_while1(is_valid_wrapped), tag("\""));

            map(parser, |v| LogCell::Str(v))(input)
        }
        _ => map(take_while1(is_valid_unwrapped), |v| LogCell::Str(v))(input),
    }
}

pub fn parse_number(input: &str) -> IResult<&str, LogCell> {
    alt((parse_multi_power, parse_float, parse_integer))(input)
}
pub fn parse_integer(input: &str) -> IResult<&str, LogCell> {
    map_res(recognize(tuple((opt(char('-')), digit1))), |s: &str| {
        s.parse::<i64>().map(|v| LogCell::Integer(v))
    })(input)
}

pub fn parse_float(input: &str) -> IResult<&str, LogCell> {
    let parser = map_res(
        recognize(tuple((
            opt(char('-')),
            digit1,
            map(tuple((char('.'), digit1)), |_| ()),
        ))),
        |s: &str| s.parse::<f64>(),
    );

    map(parser, |v| LogCell::Float(v))(input)
}

pub fn parse_multi_power(input: &str) -> IResult<&str, LogCell> {
    let parser = tuple((
        map_res(digit1, str::parse),
        tag("|"),
        map_res(digit1, str::parse),
    ));

    map(parser, |v| LogCell::MultiPowerCell { 0: (v.0, v.2) })(input)
}

pub fn is_valid_emote(c: char) -> bool {
    let cv = c as u32;
    cv >= 0x20
}

pub fn is_valid_wrapped(c: char) -> bool {
    let cv = c as u32;
    (cv >= 0x20) && (cv != 0x22) && (cv != 0x5C)
}

pub fn is_valid_unwrapped(c: char) -> bool {
    let cv = c as u32;
    (cv >= 0x20) && (cv != 0x22) && (cv != 0x5C) && (cv != 0x5D) && (cv != 0x2C) && (cv != 0x29)
}

pub fn parse_log_csv(input: &str) -> IResult<&str, LogRow> {
    let (eventtype, _) = input.split_once(",").unwrap();
    let res = match eventtype {
        "EMOTE" => {
            let (remainder, cell) = parse_emote_line(input)?;
            Ok((remainder, LogRow::Emote(cell)))
        }
        "SPELL_CAST_SUCCESS" => {
            let (remainder, cell) = parse_spell_cast_success_line(input)?;
            Ok((remainder, LogRow::SpellCastSuccess(cell)))
        }
        "SPELL_DAMAGE" => {
            let (remainder, cell) = parse_spell_damage_line(input)?;
            Ok((remainder, LogRow::SpellDamage(cell)))
        }
        "SPELL_HEAL" => {
            let (remainder, cell) = parse_spell_heal_line(input)?;
            Ok((remainder, LogRow::SpellHeal(cell)))
        }
        _ => Ok((input, LogRow::NotSupported)),
    };
    res
}

fn parse_emote_fields(
    input: &str,
) -> IResult<&str, (&str, &str, &str, &str, &str, &str, &str, &str, &str)> {
    tuple((
        take_while1(|c| c != ','),
        tag(","),
        take_while1(|c| c != ','),
        tag(","),
        take_while1(|c| c != ','),
        tag(","),
        take_while1(|c| c != ','),
        tag(","),
        recognize(not_line_ending),
    ))(input)
}

pub fn parse_emote_line(input: &str) -> IResult<&str, LogEmote> {
    map(parse_emote_fields, |emote_tuple| LogEmote {
        sourceGUID: emote_tuple.0,
        sourcename: emote_tuple.2,
        sourceflags: emote_tuple.4,
        sourceraidflags: emote_tuple.6,
        text: emote_tuple.8,
    })
    .parse(input)
}

pub fn parse_spell_cast_success_line(input: &str) -> IResult<&str, LogSpellCastSuccess> {
    let (remainder, (event, _, cols)) = tuple((
        tag("SPELL_CAST_SUCCESS"),
        tag(","),
        separated_list1(tag(","), parse_log_cell),
    ))(input)?;

    if event != "SPELL_CAST_SUCCESS" {
        return Err(Err::Error(nom::error::Error {
            input,
            code: ErrorKind::Tag,
        }));
    }

    if cols.len() != 28 {
        panic!(
            "Spell cast success event malformed. Should have 28 fields, had: {:?}. cols: {:?}. input: {:?}",
            cols.len(),
            cols,
            input
        );
    }

    let mut cols_iter = cols.into_iter();

    Ok((
        remainder,
        LogSpellCastSuccess {
            sourceGUID: cols_iter.next().unwrap(),
            sourceName: cols_iter.next().unwrap(),
            sourceFlags: cols_iter.next().unwrap(),
            sourceRaidFlags: cols_iter.next().unwrap(),
            destGUID: cols_iter.next().unwrap(),
            destName: cols_iter.next().unwrap(),
            destFlags: cols_iter.next().unwrap(),
            destRaidFlags: cols_iter.next().unwrap(),
            spellId: cols_iter.next().unwrap(),
            spellName: cols_iter.next().unwrap(),
            spellSchool: cols_iter.next().unwrap(),
            unitGUID: cols_iter.next().unwrap(),
            ownerGUID: cols_iter.next().unwrap(),
            currHp: cols_iter.next().unwrap(),
            maxHp: cols_iter.next().unwrap(),
            attackPower: cols_iter.next().unwrap(),
            spellPower: cols_iter.next().unwrap(),
            armor: cols_iter.next().unwrap(),
            totalDamageAbsorbs: cols_iter.next().unwrap(),
            resourceType: cols_iter.next().unwrap(),
            currResource: cols_iter.next().unwrap(),
            maxResource: cols_iter.next().unwrap(),
            resourceCost: cols_iter.next().unwrap(),
            y: cols_iter.next().unwrap(),
            x: cols_iter.next().unwrap(),
            mapId: cols_iter.next().unwrap(),
            facing: cols_iter.next().unwrap(),
            ilvl: cols_iter.next().unwrap(),
        },
    ))
}

pub fn parse_spell_damage_line(input: &str) -> IResult<&str, LogSpellDamage> {
    let (remainder, (event, _, cols)) = tuple((
        tag("SPELL_DAMAGE"),
        tag(","),
        separated_list1(tag(","), parse_log_cell),
    ))(input)?;

    if event != "SPELL_DAMAGE" {
        println!("Failed to parse event: {:?}", event);
        return Err(Err::Error(nom::error::Error {
            input,
            code: ErrorKind::Tag,
        }));
    }

    if cols.len() != 38 {
        panic!(
            "Spell damage event malformed. Should have 38 fields, had: {:?}. cols: {:?}. input: {:?}",
            cols.len(),
            cols,
            input
        );
    }

    let mut cols_iter = cols.into_iter();

    Ok((
        remainder,
        LogSpellDamage {
            sourceGUID: cols_iter.next().unwrap(),
            sourceName: cols_iter.next().unwrap(),
            sourceFlags: cols_iter.next().unwrap(),
            sourceRaidFlags: cols_iter.next().unwrap(),
            destGUID: cols_iter.next().unwrap(),
            destName: cols_iter.next().unwrap(),
            destFlags: cols_iter.next().unwrap(),
            destRaidFlags: cols_iter.next().unwrap(),
            spellId: cols_iter.next().unwrap(),
            spellName: cols_iter.next().unwrap(),
            spellSchool: cols_iter.next().unwrap(),
            unitGUID: cols_iter.next().unwrap(),
            ownerGUID: cols_iter.next().unwrap(),
            currHp: cols_iter.next().unwrap(),
            maxHp: cols_iter.next().unwrap(),
            attackPower: cols_iter.next().unwrap(),
            spellPower: cols_iter.next().unwrap(),
            armor: cols_iter.next().unwrap(),
            totalDamageAbsorbs: cols_iter.next().unwrap(),
            resourceType: cols_iter.next().unwrap(),
            currResource: cols_iter.next().unwrap(),
            maxResource: cols_iter.next().unwrap(),
            resourceCost: cols_iter.next().unwrap(),
            y: cols_iter.next().unwrap(),
            x: cols_iter.next().unwrap(),
            mapId: cols_iter.next().unwrap(),
            facing: cols_iter.next().unwrap(),
            ilvl: cols_iter.next().unwrap(),
            amount: cols_iter.next().unwrap(),
            overkill: cols_iter.next().unwrap(),
            school: cols_iter.next().unwrap(),
            resisted: cols_iter.next().unwrap(),
            blocked: cols_iter.next().unwrap(),
            absorbed: cols_iter.next().unwrap(),
            critical: cols_iter.next().unwrap().into(),
            glancing: cols_iter.next().unwrap().into(),
            crushing: cols_iter.next().unwrap().into(),
            isOffHand: cols_iter.next().unwrap().into(),
        },
    ))
}

pub fn parse_spell_heal_line(input: &str) -> IResult<&str, LogSpellHeal> {
    let (remainder, (event, _, cols)) = tuple((
        tag("SPELL_HEAL"),
        tag(","),
        separated_list1(tag(","), parse_log_cell),
    ))(input)?;

    if event != "SPELL_HEAL" {
        return Err(Err::Error(nom::error::Error {
            input,
            code: ErrorKind::Tag,
        }));
    }
    if cols.len() != 33 {
        println!(
            "Spell heal event malformed. Should have 33 fields, had: {:?}",
            cols.len()
        );
        return Err(Err::Error(nom::error::Error {
            input,
            code: ErrorKind::LengthValue,
        }));
    }

    let mut cols_iter = cols.into_iter();

    Ok((
        remainder,
        LogSpellHeal {
            sourceGUID: cols_iter.next().unwrap(),
            sourceName: cols_iter.next().unwrap(),
            sourceFlags: cols_iter.next().unwrap(),
            sourceRaidFlags: cols_iter.next().unwrap(),
            destGUID: cols_iter.next().unwrap(),
            destName: cols_iter.next().unwrap(),
            destFlags: cols_iter.next().unwrap(),
            destRaidFlags: cols_iter.next().unwrap(),
            spellId: cols_iter.next().unwrap(),
            spellName: cols_iter.next().unwrap(),
            spellSchool: cols_iter.next().unwrap(),
            unitGUID: cols_iter.next().unwrap(),
            ownerGUID: cols_iter.next().unwrap(),
            currHp: cols_iter.next().unwrap(),
            maxHp: cols_iter.next().unwrap(),
            attackPower: cols_iter.next().unwrap(),
            spellPower: cols_iter.next().unwrap(),
            armor: cols_iter.next().unwrap(),
            totalDamageAbsorbs: cols_iter.next().unwrap(),
            resourceType: cols_iter.next().unwrap(),
            currResource: cols_iter.next().unwrap(),
            maxResource: cols_iter.next().unwrap(),
            resourceCost: cols_iter.next().unwrap(),
            y: cols_iter.next().unwrap(),
            x: cols_iter.next().unwrap(),
            mapId: cols_iter.next().unwrap(),
            facing: cols_iter.next().unwrap(),
            ilvl: cols_iter.next().unwrap(),
            amount: cols_iter.next().unwrap(),
            overhealing: cols_iter.next().unwrap(),
            absorbed: cols_iter.next().unwrap(),
            critical: cols_iter.next().unwrap().into(),
            // Last field is always nil.
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_spell_damage_event() {
        let input = "SPELL_DAMAGE,Player-1379-0A9FF58F,\"Yerrog-Sanguino\",0x512,0x0,Creature-0-4252-2515-19964-196102-000550239A,\"Conjured Lasher\",0xa48,0x0,213709,\"Brambles\",0x8,Creature-0-4252-2515-19964-196102-000550239A,0000000000000000,1483954,1952835,0,0,5043,0,1,0,0,0,-5095.52,1142.47,2073,6.1556,70,488,488,-1,8,0,0,0,nil,nil,nil";
        parse_spell_damage_line(input).unwrap();
    }
}
