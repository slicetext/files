use std::{env, fs::{self}, path::PathBuf, process::Command};
use egui_extras::{Column, TableBuilder};
use time::{format_description, OffsetDateTime};

use eframe::egui::{self, Color32, Key, KeyboardShortcut, Modifiers, Response, Stroke};

const APP_NAME: &str = "File Explorer";

struct BookMark {
    name: String,
    path: PathBuf,
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
struct FileData {
    name: String,
    size: u64,
    path: PathBuf,
    access: String,
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
struct DirData {
    name: String,
    path: PathBuf,
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
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
                path: item.path(),
                access: match item.metadata().unwrap().accessed() {
                    Ok(a) => {
                        let format=format_description::parse("[year]-[month]-[day] [hour]:[minute]").unwrap();
                        let time: OffsetDateTime = a.into();
                        time.format(&format).unwrap()
                    }
                    Err(_) => {
                        String::from("")
                    }
                }
            }));
        }
    }
    dirs.sort();
    files.sort();
    dirs.append(&mut files);
    return dirs;
}

fn main() {
    // Eframe
    let options = eframe::NativeOptions {
        ..Default::default()
    };

    // State
    let mut path: PathBuf = env::current_dir().unwrap();
    let bookmarks: Vec<BookMark> = vec![
        BookMark {
            name: String::from("Home"),
            path: dirs::home_dir().unwrap(),
        },
        BookMark {
            name: String::from("Downloads"),
            path: dirs::download_dir().unwrap_or(dirs::home_dir().unwrap()),
        },
        BookMark {
            name: String::from("Documents"),
            path: dirs::document_dir().unwrap_or(dirs::home_dir().unwrap()),
        },
        BookMark {
            name: String::from("Pictures"),
            path: dirs::picture_dir().unwrap_or(dirs::home_dir().unwrap()),
        },
        BookMark {
            name: String::from("Music"),
            path: dirs::audio_dir().unwrap_or(dirs::home_dir().unwrap()),
        },
        BookMark {
            name: String::from("Videos"),
            path: dirs::video_dir().unwrap_or(dirs::home_dir().unwrap()),
        },
    ];
    let mut side_expanded: bool = true;
    let mut copy_buf: PathBuf = PathBuf::new();
    let mut selected_index: i64 = -1;
    let mut rename_open: bool = false;
    let mut rename_text: String = String::new();
    let mut rename_path: PathBuf = PathBuf::new();

    let _ = eframe::run_simple_native(APP_NAME, options, move |ctx, _frame| {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                if ui.button("<").clicked() {
                    path.pop();
                }
                ui.label(path.to_string_lossy());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("+").clicked() {
                        let mut new_path = path.clone();
                        new_path.push("new_file.txt");
                        if new_path.exists() {
                            let mut i = 0;
                            loop {
                                new_path.pop();
                                new_path.push("new_file_".to_owned()+&i.to_string()+".txt");
                                if !new_path.exists() {
                                    break;
                                }
                                i += 1;
                            }
                        }
                        fs::write(new_path,"").unwrap();
                    }
                    if ui.button("New Folder").clicked() {
                        let mut new_path = path.clone();
                        new_path.push("new_folder");
                        if new_path.exists() {
                            let mut i = 0;
                            loop {
                                new_path.pop();
                                new_path.push("new_folder_".to_owned()+&i.to_string());
                                if !new_path.exists() {
                                    break;
                                }
                                i += 1;
                            }
                        }
                        fs::create_dir(&new_path).unwrap();
                    }
                    if rename_open {
                        let edit = ui.text_edit_singleline(&mut rename_text);
                        if edit.lost_focus() {
                            let mut new_path = rename_path.clone();
                            new_path.pop();
                            new_path.push(&rename_text);
                            if new_path.exists() {
                                let mut i = 0;
                                loop {
                                    new_path.pop();
                                    new_path.push(rename_text.clone()+&i.to_string());
                                    if !new_path.exists() {
                                        break;
                                    }
                                    i += 1;
                                }
                            }
                            fs::rename(&rename_path, &new_path).unwrap();
                            rename_text = String::new();
                            rename_open = false;
                        }
                        edit.request_focus();
                    }
                });
            });
        });
        let toggle_marks = KeyboardShortcut::new(Modifiers {
            ctrl: true,
            alt: false,
            shift: false,
            command: false,
            mac_cmd: false,
        }, Key::Tab);
        let copy = KeyboardShortcut::new(Modifiers {
            ctrl: true,
            alt: false,
            shift: false,
            command: false,
            mac_cmd: false,
        }, Key::Y);
        let paste = KeyboardShortcut::new(Modifiers {
            ctrl: true,
            alt: false,
            shift: false,
            command: false,
            mac_cmd: false,
        }, Key::P);
        let delete = KeyboardShortcut::new(Modifiers {
            ctrl: true,
            alt: false,
            shift: false,
            command: false,
            mac_cmd: false,
        }, Key::Backspace);
        let rename = KeyboardShortcut::new(Modifiers {
            ctrl: false,
            alt: false,
            shift: false,
            command: false,
            mac_cmd: false,
        }, Key::F2);

        ctx.input_mut(|i| {
            // Toggle side bar
            if i.consume_shortcut(&toggle_marks) {
                side_expanded = !side_expanded;
            }
        });
        egui::SidePanel::left("bookmarks")
            .resizable(true)
            .show_animated(ctx, side_expanded, |ui| {
                TableBuilder::new(ui)
                    .column(Column::remainder())
                    .body(|mut body| {
                        for i in &bookmarks {
                            body.row(30.0, |mut row| {
                                row.col(|ui| {
                                    if ui.button(&i.name).clicked() {
                                        path = i.path.to_path_buf();
                                    }
                                });
                            });
                        }
                    });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                TableBuilder::new(ui)
                    .column(Column::remainder())
                    .column(Column::initial(140.0))
                    .column(Column::initial(200.0))
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("Name");
                        });
                        header.col(|ui| {
                            ui.heading("Size");
                        });
                        header.col(|ui| {
                            ui.heading("Date");
                        });
                    })
                    .body(|mut body| {
                        let items = get_dir(&path.to_str().unwrap());
                        ctx.input_mut(|i| {
                            if i.consume_shortcut(&copy) {
                                println!("copy");
                                copy_buf = match &items[selected_index as usize] {
                                    ItemData::Dir(d) => {
                                        d.path.clone()
                                    },
                                    ItemData::File(d) => {
                                        d.path.clone()
                                    }

                                }
                            } else if i.consume_shortcut(&paste) {
                                let mut t_path = path.clone();
                                t_path.push(copy_buf.file_name().unwrap());
                                fs::copy(&copy_buf, t_path).unwrap();
                            }
                            if i .consume_shortcut(&delete) {
                                match &items[selected_index as usize] {
                                    ItemData::Dir(d) => {
                                        fs::remove_dir_all(&d.path).unwrap();
                                    },
                                    ItemData::File(d) => {
                                        fs::remove_file(&d.path).unwrap();
                                    }
                                };
                            }
                            if i.consume_shortcut(&rename) {
                                let path = match &items[selected_index as usize] {
                                    ItemData::Dir(d)  => d.path.clone(),
                                    ItemData::File(d) => d.path.clone(),
                                };
                                rename_path = path;
                                rename_open = true;
                            }
                        });
                        for index in 0..items.len() {
                            let i = &items[index];
                            body.row(30.0, |mut row| {
                                match i {
                                    ItemData::Dir(d) => {
                                        row.col(|ui| {
                                            let label: Response;
                                            if index as i64 == selected_index {
                                                label = ui.add(egui::Button::new(format!("{}",d.name)).stroke(Stroke {
                                                    width: 3.0,
                                                    color: Color32::from_rgb(31, 196, 211)
                                                }));
                                            } else {
                                                label = ui.button(format!("{}",d.name));
                                            }
                                            if label.double_clicked() {
                                                path = d.path.clone().into();
                                                selected_index = -1;
                                            } else if label.clicked() {
                                                selected_index = index as i64;
                                            }
                                        });
                                    },
                                    ItemData::File(d) => {
                                        row.col(|ui| {
                                            let label: Response;
                                            if index as i64 == selected_index {
                                                label = ui.add(egui::Button::new(format!("{}",d.name)).stroke(Stroke {
                                                    width: 3.0,
                                                    color: Color32::from_rgb(31, 196, 211)
                                                }));
                                            } else {
                                                label = ui.button(format!("{}",d.name));
                                            }
                                            if label.double_clicked() {
                                                let _ = Command::new("xdg-open")
                                                    .arg(d.path.to_str().unwrap())
                                                    .spawn()
                                                    .unwrap();
                                            } else if label.clicked() {
                                                selected_index = index as i64;
                                            }
                                        });
                                        row.col(|ui| {
                                            ui.label(format!("{}",d.size));
                                        });
                                        row.col(|ui| {
                                            ui.label(format!("{}",d.access));
                                        });
                                    }
                                };
                            });
                        }
                    })
            })
        });
    });
}
