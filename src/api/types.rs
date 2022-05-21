use std::fmt::{Display, Formatter};

use actix_web::{body::BoxBody, HttpRequest, HttpResponse, Responder};
use r2d2_sqlite::{rusqlite, rusqlite::{ToSql, types::{FromSql, ToSqlOutput}}};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Validate)]
pub(crate) struct BeckonCowsRequest {
    #[validate(range(min = 1, max = 5))] // custom input validation macro
    pub count: u32,
}

#[derive(Debug, Serialize)]
pub(crate) struct CowListResponse {
    pub cows: Vec<Cow>,
}

impl Display for CowListResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = self.cows.iter().map(|c| format!("{}", c));
        let v: Vec<String> = s.collect();
        let p = v.join(", ");
        write!(f, "BeckonCowsResponse {{ cows: [{}] }}", p)
    }
}

impl Responder for CowListResponse {
    type Body = BoxBody;
    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string_pretty(&self).unwrap();
        HttpResponse::Ok()
            .content_type("application/json")
            .body(body)
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct Cow {
    pub name: String,
    pub id: u32,
    pub color: CowColor,
    pub age: u32,
    pub weight: u32,
}

impl Cow {
    pub fn new(name: impl AsRef<str>, id: u32, color: CowColor, age: u32, weight: u32) -> Self {
        Cow { name: String::from(name.as_ref()), id, color, age, weight }
    }
}

impl Display for Cow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "a cow named {} (id {}), {}, {} years old and weighs {} pounds",
                self.name, self.id, self.color.as_ref(), self.age, self.weight)
    }
}

#[derive(Debug, Serialize)]
pub(crate) enum CowColor {
    Black, Brown, Tan, BlackWithWhitePatches, 
}

impl AsRef<str> for CowColor {
    fn as_ref(&self) -> &str {
        match self {
            CowColor::Black => "black",
            CowColor::Brown => "brown",
            CowColor::Tan => "tan",
            CowColor::BlackWithWhitePatches => "black and white patches",
        }
    }
}

impl TryFrom<&str> for CowColor {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.as_ref() {
            "black" => Ok(CowColor::Black),
            "brown" => Ok(CowColor::Brown),
            "tan" => Ok(CowColor::Tan),
            "black and white patches" => Ok(CowColor::BlackWithWhitePatches),
            _ => Err(()),
        }
    }
}

impl ToSql for CowColor {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        rusqlite::Result::Ok(ToSqlOutput::from(self.as_ref()))
    }
}

impl FromSql for CowColor {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let s: String = FromSql::column_result(value)?;
        match CowColor::try_from(s.as_str()) {
            Ok(c) => Ok(c),
            Err(_) => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}
