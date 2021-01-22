mod settings;

use log;
use notify::{watcher, RecursiveMode, Watcher};
use pulldown_cmark::{html, Options, Parser};
use regex::Regex;
use serde::Serialize;
use settings::Settings;
use simple_logger;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;
use tera::{Context, Tera};

#[derive(Debug, Serialize)]
struct Link {
    title: String,
    target: String,
}

#[derive(Debug, Serialize)]
struct Note {
    title: String,
    path: String,
    id: String,
    // date: String,
    // tasks: Vec<Task>,
    links: Vec<Link>,
    contents: String,
}

fn parse_note(path: &str) -> Note {
    let mut file = File::open(path).expect("lol");
    let path = Path::new(path).file_name().unwrap().to_str().unwrap();
    let id = &String::from(path)[..10];

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let title_regex = Regex::new(r"^# (.+)").unwrap();
    let title = &title_regex.captures(&contents).unwrap()[1];

    let links_regex = Regex::new(r"(?m)\[(?P<desc>.*)\]\(notes:(?P<id>.+)\)").unwrap();
    let mut links = Vec::new();
    for link in links_regex.captures_iter(&contents) {
        links.push(Link {
            title: String::from(&link[1]),
            target: String::from(&link[2]),
        });
    }
    let contents = links_regex.replace_all(&contents, "[$desc](/$id.html)");

    Note {
        title: String::from(title),
        id: String::from(id),
        path: String::from(path),
        links,
        contents: String::from(contents),
    }
}

fn main() {
    simple_logger::SimpleLogger::new()
        .init()
        .expect("Logger failed to initialize");

    let mut tera = match Tera::new("templates/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    tera.autoescape_on(vec![]);

    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(10)).unwrap();

    let settings = Settings::new().unwrap();
    log::info!("settings: {:?}", settings);
    watcher
        .watch(&settings.directory, RecursiveMode::Recursive)
        .unwrap();

    loop {
        match rx.recv() {
            Ok(event) => {
                log::info!("{:?}", event);
                let mut notes = HashMap::new();
                for file in std::fs::read_dir(&settings.directory).unwrap() {
                    let file = file.unwrap();
                    let path = String::from(file.path().to_str().unwrap());
                    if path.ends_with(".md") {
                        let note = parse_note(&path);
                        notes.insert(String::from(note.id.to_string()), note);
                    }
                }

                let mut context = Context::new();
                let serialized = serde_json::to_string(&notes).unwrap();
                context.insert("notes", &serialized);
                context.insert("notes_dict", &notes);
                context.insert("index", &settings.index);
                let result = match tera.render("random.html", &context) {
                    Ok(t) => t,
                    Err(error) => panic!("{:?}", error),
                };
                let mut file = File::create("output/random.html").unwrap();
                file.write_all(result.as_bytes()).unwrap();
                let result = match tera.render("search.html", &context) {
                    Ok(t) => t,
                    Err(error) => panic!("{:?}", error),
                };
                let mut file = File::create("output/search.html").unwrap();
                file.write_all(result.as_bytes()).unwrap();

                for (id, note) in notes {
                    let mut options = Options::empty();
                    options.insert(Options::ENABLE_STRIKETHROUGH);
                    let parser = Parser::new_ext(&note.contents, options);
                    let mut contents = String::new();
                    html::push_html(&mut contents, parser);

                    let mut context = Context::new();
                    context.insert("title", &note.title);
                    context.insert("contents", &contents);
                    let result = match tera.render("note.html", &context) {
                        Ok(t) => t,
                        Err(error) => panic!("{:?}", error),
                    };

                    let id = if id == settings.index {
                        String::from("index")
                    } else {
                        id
                    };

                    let mut file = File::create(format!("output/{}.html", id)).unwrap();
                    file.write_all(result.as_bytes()).unwrap();
                    let serialized = serde_json::to_string(&note).unwrap();
                    let mut file = File::create(format!("output/{}.json", id)).unwrap();
                    file.write_all(serialized.as_bytes()).unwrap();
                    log::info!("rerun web export of slipbox notes");
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
