use std::io;
use std::io::Read;
use std::process::{Command, Stdio};
use regex::Regex;
use crate::information_meta::ProjectType;

pub fn get_main_meta(project_type: ProjectType, name: String) -> String {
    match project_type {
        ProjectType::DefaultExecutable => {
            r"#include <iostream>

namespace %0% {
template<typename T> auto do_some(const T& out) -> void {
    std::cout << out << std::endl;
    }
}

auto main() -> int {
    %0%::do_some(104);
}

            ".to_owned().replace("%0%", name.replace("-", "_").as_str())
        }
        ProjectType::DefaultLibrary => {
            "".to_owned()
        }
    }
}

pub fn get_cmake_meta(project_type: ProjectType ,name: String, cxx_version: u8) -> io::Result<String> {
    match project_type {
        ProjectType::DefaultExecutable => {
            let meta = r"cmake_minimum_required(VERSION 3.25)
project(%0%)

set(CMAKE_CXX_STANDARD %1%)

set(SOURCE_DIR source_files)
set(INCLUDE_DIR include_files)

include_directories(${INCLUDE_DIR})
file(GLOB SOURCE_FILES ${SOURCE_DIR}/*.cpp ${SOURCE_DIR}/*.c)

add_executable(%0% ${SOURCE_FILES})
                ".to_owned()
                .replace("%0%", name.replace("-", "_").as_str())
                .replace("%1%", cxx_version.to_string().as_str())
                .replace("%2%", get_cmake_version()?.as_str());
            Ok(meta)
        }
        ProjectType::DefaultLibrary => {
            Ok("".to_owned())
        }
    }
}

pub fn get_cmake_version() -> io::Result<String> {
    let command = Command::new("cmake").stdout(Stdio::piped()).arg("--version").spawn()?;
    let mut out = command.stdout.unwrap();
    let mut buf = String::new();
    let _ = out.read_to_string(&mut buf).expect("unable to init buf!");
    let regex = Regex::new(r"cmake version (\d+\.\d+\.\d+)").unwrap();
    Ok(regex
        .captures(&buf)
        .expect("unable to get cmake version!")
        .get(1)
        .expect("unable to get cmake version!")
        .as_str()
        .to_owned()
    )
}