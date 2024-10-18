use std::io;
use std::io::Read;
use std::process::{Command, Stdio};

pub struct ConanTerminal;

#[derive(PartialEq)]
pub enum ConanDependencyVersion{
    None,
    Version(String),
}

pub struct ConanDependency {
    pub (crate) name: String,
    pub (crate) version: ConanDependencyVersion
}

impl ConanDependency {
    pub fn new(name: String, version: ConanDependencyVersion) -> Self {
        Self {
            name,
            version,
        }
    }

    pub fn load_latest_version(&self) -> io::Result<String> {
        let command = Command::new("conan")
            .stdout(Stdio::piped())
            .args(["search", "boost", "--remote=conancenter"])
            .spawn()?;
        let mut out = command.stdout.unwrap();
        let mut buf = String::new();
        let _ = out.read_to_string(&mut buf);
        Ok(buf.trim_end()
            .to_string()
            .split('\n')
            .map(|item| item.to_string())
            .collect::<Vec<String>>()
            .last()
            .unwrap()
            .clone()
        )
    }
}