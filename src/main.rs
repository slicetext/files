use std::{fmt::format, fs::{self}, path::PathBuf};

use eframe::{egui, run_simple_native};

const APP_NAME: &str = "File Explorer";

struct FileData {
    name: String,
    size: u64,
}

struct DirData {
    name: String,
    path: PathBuf,
}

enum ItemData {
    Dir(DirData),
    File(FileData),
}

fn get_dir(path: &str) -> Vec<ItemData> {
    let mut files: Vec<ItemData> = vec![];
    let mut dirs: Vec<ItemData> = vec![];
    let dir = fs::read_dir(path).unwrap();
    for i in dir {
        let item = i.unwrap();
        if item.metadata().unwrap().is_dir() {
            dirs.push(ItemData::Dir(DirData {
                name: item.file_name().into_string().unwrap(),
                path: item.path(),
            }));
        } else {
            files.push(ItemData::File(FileData {
                name: item.file_name().into_string().unwrap(),
                size: item.metadata().unwrap().len(),
            }));
        }
    }
    dirs.append(&mut files);
    return dirs;
}

fn main() {
    // Eframe
    let options = eframe::NativeOptions {
        ..Default::default()
    };

    // State
    let mut path: String = String::from(".");

    let _ = eframe::run_simple_native(APP_NAME, options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            for i in get_dir(&path) {
                match i {
                    ItemData::Dir(d) => {
                        if ui.button(format!("{}",d.name)).clicked() {
                            path = d.path.to_str().unwrap().to_owned();
                        }
                    },
                    ItemData::File(d) => {
                        ui.label(format!("{} -- {}",d.name, d.size));
                    }
                };
            }
        });
    });
}
