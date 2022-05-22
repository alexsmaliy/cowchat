use std::fmt::{
    Display, Formatter,
};

use actix_web::{
    body::BoxBody, HttpRequest, HttpResponse, Responder,
};
use r2d2_sqlite::{
    rusqlite,
    rusqlite::{
        ToSql,
        types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
    },
};
use serde::{
    Deserialize, Serialize,
};
use validator::Validate;

// Derive directives create minimal automatic implementations of certain fundamental traits.
// Deserialize is about unmarshalling values from JSON sent over the wire.
#[derive(Deserialize, Validate)]
pub(crate) struct BeckonCowsRequest {
    #[validate(range(min = 1, max = 5))] // library-provided input validation macro
    pub count: u32,
}

// The Debug trait is for pretty-printing values using the debug string formatter `{:?}`.
// Serialize is about marshalling values into JSON to send over the wire.
#[derive(Debug, Serialize)]
pub(crate) struct CowListResponse {
    pub cows: Vec<Cow>,
}

// Display can't be auto-implemented in plain Rust, but there are custom traits for it.
// Display is about the to-string behavior of a type.
impl Display for CowListResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str_iter = self.cows.iter().map(|c| format!("{}", c));
        let str_vec: Vec<String> = str_iter.collect();
        let one_string = str_vec.join(", ");
        write!(f, "BeckonCowsResponse {{ cows: [{}] }}", one_string)
    }
}

// Responder is a trait for Actix to denote things that can be sent as HTTP response bodies.
impl Responder for CowListResponse {
    // A trait can define an associated inner type that implementations must specify.
    // This is similar to, but distinct from a generic type variable. One trait
    // can't be implemented for the same type multiple times with different associated
    // types. However, the same trait can be implemented multiple times with
    // different generic types. For example, u32 can be Add<u32> and Add<String>
    // and have different behavior for each one, like addition and concatenation.
    type Body = BoxBody;
    
    // Unused argument names should be prefaced with `_` for readability.
    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string_pretty(&self).unwrap();
        HttpResponse::Ok()
            .content_type("application/json")
            .body(body)
    }
}

// All the fields are public, because we want to be able to destructure this type elsewhere.
#[derive(Debug, Serialize)]
pub(crate) struct Cow {
    pub name: String,
    pub id: u32,
    pub color: CowColor,
    pub age: u32,
    pub weight: u32,
}

// We give Cow a constructor for convenience, but it can also be constructed as Cow { ...fields... }.
impl Cow {
    pub fn new(name: &str, id: u32, color: CowColor, age: u32, weight: u32) -> Self {
        Self { name: String::from(name), id, color, age, weight }
    }
}

impl Display for Cow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "a cow named {} (id {}), {}, {} years old and weighs {} pounds",
                self.name, self.id, self.color.as_ref(), self.age, self.weight)
    }
}

// The simplest knd of enum is just a finite list of literal instances.
// Enums can also be other kinds of type unions.
#[derive(Debug, Serialize)]
pub(crate) enum CowColor {
    Black, Brown, Tan, BlackWithWhitePatches, 
}

// This makes it possible to get a `&str` out of a CowColor.
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

// This makes it possible to get a CowColor out of a `&str`.
impl TryFrom<&str> for CowColor {
    // The TryFrom trait defines an associated type denoting the flavor of error
    // thrown during unsuccessful conversion. Anyhow is a library for simplifying
    // error handling.
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "black" => Ok(CowColor::Black),
            "brown" => Ok(CowColor::Brown),
            "tan" => Ok(CowColor::Tan),
            "black and white patches" => Ok(CowColor::BlackWithWhitePatches),
            x => Err(anyhow::anyhow!("{} is not a valid CowColor!", x)),
        }
    }
}

impl ToSql for CowColor {
    // '_ is the anonymous reference lifetime, used where a lifetime annotation is
    // required (e.g., in function declarations), but can be reasonably inferred.
    // Types that might contain references as internal fields must be annotated
    // with lifetimes.
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        rusqlite::Result::Ok(ToSqlOutput::from(self.as_ref()))
    }
}

impl FromSql for CowColor {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s: String = FromSql::column_result(value)?;
        // &*String is equivalent to String.as_str(), because String is Deref<Target = str>.
        // Note that Deref::Target is the associated type of Deref, not a generic parmeter.
        // Any given type can only dereference into a single other type, which is one thing
        // that associated types enforce and generic types do not.
        match CowColor::try_from(&*s) {
            Ok(c) => Ok(c),
            Err(_) => Err(FromSqlError::InvalidType),
        }
    }
}
