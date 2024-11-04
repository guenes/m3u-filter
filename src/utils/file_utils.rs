use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use log::{debug, error};
use path_absolutize::Absolutize;

const USER_FILE: &str = "user.txt";
const CONFIG_PATH: &str = "config";
const CONFIG_FILE: &str = "config.yml";
const SOURCE_FILE: &str = "source.yml";
const MAPPING_FILE: &str = "mapping.yml";
const API_PROXY_FILE: &str = "api-proxy.yml";

#[macro_export]
macro_rules! exit {
    ($($arg:tt)*) => {{
        error!($($arg)*);
        std::process::exit(1);
    }};
}

pub fn get_exe_path() -> PathBuf {
    let default_path = std::path::PathBuf::from("./");
    let current_exe = std::env::current_exe();
    match current_exe {
        Ok(exe) => {
            match fs::read_link(&exe) {
                Ok(f) => f.parent().map_or(default_path, std::path::Path::to_path_buf),
                Err(_) => return exe.parent().map_or(default_path, std::path::Path::to_path_buf)
            }
        }
        Err(_) => default_path
    }
}

fn get_default_path(file: &str) -> String {
    let path: PathBuf = get_exe_path();
    let default_path = path.join(file);
    String::from(if default_path.exists() {
        default_path.to_str().unwrap_or(file)
    } else {
        file
    })
}

pub fn get_default_file_path(config_path: &str, file: &str) -> String {
    let path: PathBuf = PathBuf::from(config_path);
    let default_path = path.join(file);
    String::from(if default_path.exists() {
        default_path.to_str().unwrap_or(file)
    } else {
        file
    })
}

#[inline]
pub fn get_default_user_file_path(config_path: &str) -> String {
    get_default_file_path(config_path, USER_FILE)
}

#[inline]
pub fn get_default_config_path() -> String {
    get_default_path(CONFIG_PATH)
}

#[inline]
pub fn get_default_config_file_path(config_path: &str) -> String {
    get_default_file_path(config_path, CONFIG_FILE)
}

#[inline]
pub fn get_default_sources_file_path(config_path: &str) -> String {
    get_default_file_path(config_path, SOURCE_FILE)
}

#[inline]
pub fn get_default_mappings_path(config_path: &str) -> String {
    get_default_file_path(config_path, MAPPING_FILE)
}

#[inline]
pub fn get_default_api_proxy_config_path(config_path: &str) -> String {
    get_default_file_path(config_path, API_PROXY_FILE)
}

pub fn get_working_path(wd: &String) -> String {
    let current_dir = std::env::current_dir().unwrap();
    if wd.is_empty() {
        String::from(current_dir.to_str().unwrap_or("."))
    } else {
        let work_path = std::path::PathBuf::from(wd);
        let _ = fs::create_dir_all(&work_path);
        let wdpath = fs::metadata(&work_path).map_or(None, |md| if md.is_dir() && !md.permissions().readonly() {
            work_path.canonicalize().ok()
        } else {
            error!("Path not found {:?}", &work_path);
            None
        });
        let rp: PathBuf = wdpath.map_or_else(|| current_dir.join(wd), |d| d);
        rp.canonicalize().map_or_else(|_| {
            error!("Path not found {:?}", &rp);
            String::from("./")
        }, |ap| String::from(ap.to_str().unwrap_or("./")))
    }
}

#[inline]
pub fn open_file(file_name: &Path) -> Result<File, std::io::Error> {
    File::open(file_name)
}

pub fn persist_file(persist_file: Option<PathBuf>, text: &String) {
    if let Some(path_buf) = persist_file {
        let filename = &path_buf.to_str().unwrap_or("?");
        match File::create(&path_buf) {
            Ok(mut file) => match file.write_all(text.as_bytes()) {
                Ok(()) => debug!("persisted: {}", filename),
                Err(e) => error!("failed to persist file {}, {}", filename, e)
            },
            Err(e) => error!("failed to persist file {}, {}", filename, e)
        }
    }
}

pub fn prepare_persist_path(file_name: &str, date_prefix: &str) -> PathBuf {
    let now = chrono::Local::now();
    let persist_filename = file_name.replace("{}", format!("{date_prefix}{}", now.format("%Y%m%d_%H%M%S").to_string().as_str()).as_str());
    std::path::PathBuf::from(persist_filename)
}

pub fn get_file_path(wd: &str, path: Option<PathBuf>) -> Option<PathBuf> {
    path.map(|p| if p.is_relative() {
                let pb = PathBuf::from(wd);
                match pb.join(&p).absolutize() {
                    Ok(os) => PathBuf::from(os),
                    Err(e) => {
                        error!("path is not relative {:?}", e);
                        p
                    }
                }
            } else {
                p
            })
}

pub fn add_prefix_to_filename(path: &Path, prefix: &str, ext: Option<&str>) -> PathBuf {
    let file_name = path.file_name().unwrap_or_default();
    let new_file_name = format!("{}{}", prefix, file_name.to_string_lossy());
    let result = path.with_file_name(new_file_name);
    match ext {
        None => result,
        Some(extension) => result.with_extension(extension)
    }
}

pub fn path_exists(file_path: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(file_path) {
        return metadata.is_file();
    }
    false
}

pub fn check_write(res: &std::io::Result<()>) -> Result<(), std::io::Error> {
    match res {
        Ok(()) => Ok(()),
        Err(_) => Err(std::io::Error::new(std::io::ErrorKind::Other, "Unable to write file")),
    }
}

pub fn append_extension(path: &Path, ext: &str) -> PathBuf {
    let extension = path.extension().map(|ext| ext.to_str().unwrap_or(""));
    path.with_extension(format!("{}{ext}", &extension.unwrap_or_default()))
}


#[inline]
pub fn sanitize_filename(file_name: &str) -> String {
    file_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect()
}