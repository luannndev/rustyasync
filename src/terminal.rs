use std::io;
use std::io::{Error, ErrorKind, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use chrono::Local;
use colored::Colorize;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::conan::{ConanDependency, ConanDependencyVersion};
use crate::information_meta::{DependencyInformation, ProjectMetaInformation, ProjectType};
use crate::meta;
use crate::terminal::JToolOrder::{CreateProject, RepairProject};

pub struct JToolTerminal;

pub enum JToolOrder {
    /* @0 - name, @1 - location, @2 - version, @3 - author */
    CreateProject(String, PathBuf, Option<String>, Option<String>),
    /* @0 - location */
    RepairProject(PathBuf),
    None
}

impl JToolTerminal {
    pub async fn handle_arguments(arguments: Vec<String>) -> io::Result<JToolOrder> {
        let base = match arguments
            .get(1) {
            None => { return Ok(JToolOrder::None); }
            Some(some) => { some }
        };
        if base.eq(".") {
            let path = PathBuf::from(arguments
                .get(0)
                .expect("unexpected error while getting the path of the executable!").replace("/rustyasync", ""));
            let result = check_path(&path, &arguments).await?;
            Ok(handle_result(result, &arguments, path).await)
        } else if base.eq("add") {
            let dependency_name = match arguments.get(2) {
                None => { return Err(Error::from(ErrorKind::UnexpectedEof)) }
                Some(name) => { name }
            };
            let version = get_argument("version".to_owned(), &arguments).await;
            let github = get_argument("remote".to_owned(), &arguments).await;
            let location = get_argument("path".to_owned(), &arguments).await;
            let conan_dependency = ConanDependency::new(dependency_name.clone(), match version {
                None => { ConanDependencyVersion::None }
                Some(some) => { ConanDependencyVersion::Version(some)}
            });
            Self::add_dependency_to_project(
                location
                    .unwrap_or(arguments.get(0)
                        .expect("unexpected error while getting the path of the executable!")
                        .replace("/rustyasync", "")),
                conan_dependency,
                github.unwrap_or("false".to_owned()),
            ).await?;
            Ok(JToolOrder::None)
        } else if base.eq("reload") {
            let path = get_argument(
                "path".to_owned(), &arguments)
                .await
                .unwrap_or(arguments.get(0)
                    .expect("unexpected error while getting the path of the executable!")
                    .replace("/rustyasync", ""));
            Self::reload_project_dependencies(path).await?;
            Ok(JToolOrder::None)
        } else {
            let path = PathBuf::from(base);
            let result = check_path(&path, &arguments).await?;
            Ok(handle_result(result, &arguments, path).await)
        }
    }

    pub async fn add_dependency_to_project(location: String, mut dependency: ConanDependency, github: String) -> io::Result<()> {
        if github.eq("false") {
            if dependency.version == ConanDependencyVersion::None {
                dependency.version = ConanDependencyVersion::Version(dependency.load_latest_version().expect("unable to load latest version!"))
            }
            let mut file = File::open(location + "/project.json").await?;
            let mut buf = String::new();
            buf = buf.trim_end().to_owned();
            let _ = file.read_to_string(&mut buf).await?;
            let mut project = serde_json::from_str::<ProjectMetaInformation>(buf.as_str())?;
            let dependency_information = DependencyInformation {
                name: dependency.name.clone(),
                version: match dependency.version {
                    ConanDependencyVersion::None => { "none".to_owned() }
                    ConanDependencyVersion::Version(some) => { some }
                },
                remote: "conancenter".to_string(),
            };
            project.dependencies.push(dependency_information);
            file.write_all(buf.as_bytes()).await?;

        }
        unimplemented!("todo implementation for github")
    }

    pub async fn reload_project_dependencies(location: String) -> io::Result<()> {
        let mut project_file = File::open(format!("{}/project.json", location)).await?;
        let mut conan_file = File::open(format!("{}/conanfile.txt", location)).await?;

        let mut buf = String::new();
        let _ = project_file.read_to_string(&mut buf);
        buf = buf.trim_end().to_owned();
        let project = serde_json::from_str::<ProjectMetaInformation>(buf.as_str())?;
        let mut raw_dependency_list = String::new();
        for dependency in &project.dependencies {
            if dependency.remote.eq("conancenter") {
                raw_dependency_list.push_str(&format!("{0}/{1}\n", dependency.name, dependency.version));
            }
        }
        let _ = conan_file.write_all(raw_dependency_list.as_bytes()).await?;

        let command = Command::new("conan")
            .stdout(Stdio::piped())
            .args(["install", location.as_str(), "--build=missing"])
            .spawn()?;
        let mut out = command.stdout.ok_or(Error::from(ErrorKind::UnexpectedEof))?;
        let mut buf = String::new();
        let _ = out.read_to_string(&mut buf).expect("unable to read log from out stream!");
        buf = buf.trim_end().to_owned();
        let mut log_file = File::create(Local::now().format("%d/%m/%Y-%H_%M_%S_log").to_string()).await?;
        let _ = log_file.write_all(buf.as_bytes()).await?;
        crate::trace::Trace::info(format!("The project dependencies have been {} reloaded!", "successfully".bright_green())).await;
        Ok(())
    }

    pub async fn create_project(name: String, location: PathBuf, version: Option<String>, author: Option<String>) -> io::Result<()> {
        let location_string = tokio::fs::canonicalize(&location.to_str().ok_or(Error::from(ErrorKind::Other))?).await?.to_str().ok_or(Error::from(ErrorKind::Other))?.to_owned();
        let project_meta = PathBuf::from(location_string.clone() + "/project.json");
        println!("{}", project_meta.to_str().unwrap());
        let mut project_meta = File::create(project_meta).await?;

        let project_information = ProjectMetaInformation::new(name,
                                                              version.unwrap_or("1.0.1".to_owned()),
                                                              author.unwrap_or("unknown".to_owned())
        );

        let to_string: io::Result<String> = match serde_json::to_string(&project_information) {
            Ok(string) => { Ok(string) }
            Err(_) => { Err(Error::from(ErrorKind::Other)) }
        };
        project_meta.write(to_string?.as_bytes()).await?;

        let directories = [PathBuf::from(location_string.clone() + "/source_files"),
            PathBuf::from(location_string.clone() + "/include_files"),
            PathBuf::from(location_string.clone() + "/build"),
            PathBuf::from(location_string.clone() + "/logs"),
            PathBuf::from(location_string.clone() + "/bin")
        ];
        for directory in &directories {
            tokio::fs::create_dir(directory).await?;
        }

        Self::create_cmake(location_string.clone(), &project_information).await?;
        Self::create_project_files(location_string.clone(), &project_information).await?;

        let log_output = format!("created {0} from {1}", &project_information.name.cyan(), &project_information.author.cyan());
        crate::trace::Trace::info(log_output).await;
        Ok(())

    }

    pub async fn repair_project(_: PathBuf) {

    }

    async fn create_cmake(location: String, project: &ProjectMetaInformation) -> io::Result<()> {
        let mut cmake_file = File::create(location + "/CMakeLists.txt").await?;
        let _ = cmake_file.write(meta:: get_cmake_meta(ProjectType::DefaultExecutable, project.name.clone(), 20)?.as_bytes()).await?;
        Ok(())
    }

    async fn create_project_files(location: String, project: &ProjectMetaInformation) -> io::Result<()> {
        let _ = File::create(format!("{}/conanfile.txt", location)).await?;
        let mut main_file = File::create(location + "/source_files/main.cpp").await?;
        main_file.write(meta::get_main_meta(ProjectType::DefaultExecutable, project.name.clone()).as_bytes()).await?;
        Ok(())
    }
}

pub async fn get_argument(argument: String, from: &Vec<String>) -> Option<String> {
    let argument = format!("--{}=", argument);
    for item in from {
        if item.starts_with(argument.as_str()) {
            return Some(item.replace(argument.as_str(), ""));
        }
    }
    None
}

/* true=new, false=handle */
async fn check_path(path: &PathBuf, _: &Vec<String>) -> io::Result<bool> {
    if path.exists() {
        if PathBuf::from(path
            .to_str()
            .ok_or(Error::new(ErrorKind::UnexpectedEof, ""))?.to_owned() + "/project.xml").exists()
        { return Ok(false); }
        Ok(true)
    } else {
        Ok(true)
    }
}

async fn handle_result(result: bool, arguments: &Vec<String>, path: PathBuf) -> JToolOrder {
    if result {
        let mut name: Option<String> = None;
        for argument in arguments {
            if argument.starts_with("--name=") {
                name = Some(argument.replace("--name=", ""))
            }
        }
        match name {
            None => {
                CreateProject("demo".to_owned(), path, None, None)
            }
            Some(name) => {
                CreateProject(name, path, None, None)
            }
        }

    } else {
        RepairProject(path)
    }
}
