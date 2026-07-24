use std::ops;
use crate::models::mirror::Mirror;

pub trait LowerCaseStartsWith {
    fn starts_with_lower_case(&self, other: &str) -> bool;
}

impl LowerCaseStartsWith for String {
    fn starts_with_lower_case(&self,other: &str) -> bool {
        self.to_lowercase().starts_with(other)
    }
}

pub trait ParseWithMirror {
    fn parse_mirror(&self, mirror: &Mirror) -> String;
}

impl ParseWithMirror for &'static str {
    fn parse_mirror(&self, mirror: &Mirror) -> String{
        mirror.parse_url(&self.to_string())
    }
}

impl ParseWithMirror for String {
    fn parse_mirror(&self, mirror: &Mirror) -> String {
        mirror.parse_url(&self)
    }
}