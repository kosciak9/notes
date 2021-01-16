extern crate regex;

use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

// parse one note- subset of org file:
// - title
// - date of creation
// - list of tasks with due dates
// - list of links to other notes [[notes::$ID][title]]

#[derive(Debug)]
struct Link {
    title: String,
    target: String,
}

#[derive(Debug)]
struct Task {
    title: String,
    status: String,
}

#[derive(Debug)]
struct Note {
    path: String,
    id: String,
    title: String,
    date: String,
    tasks: Vec<Task>,
    links: Vec<Link>,
    contents: String,
}

fn parse_org_file(path: &str) -> Result<Note, ()> {
    let mut file = File::open(path).expect("test");
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let id_regex =
        Regex::new(r"(\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b)")
            .unwrap();
    let id = &id_regex.captures(&path).unwrap()[1];

    let title_regex = Regex::new(r"^#\+TITLE: (.+)").unwrap();
    let title = &title_regex.captures(&contents).unwrap()[1];

    let date_regex = Regex::new(r"(?m)^#\+DATE: (.+)").unwrap();
    let date = &date_regex.captures(&contents).unwrap()[1];

    let tasks_regex = Regex::new(r"(?m)^\*+\s+([A-Z]+)\s+(.+)").unwrap();
    let mut tasks = Vec::new();
    for task in tasks_regex.captures_iter(&contents) {
        tasks.push(Task {
            title: String::from(&task[2]),
            status: String::from(&task[1]),
        })
    }

    let links_regex = Regex::new(r"(?m)\[\[notes:(.+)\](\[(.+)\])?]").unwrap();
    let mut links = Vec::new();
    for link in links_regex.captures_iter(&contents) {
        let title = match link.get(2) {
            Some(title) => String::from(title.as_str()),
            _ => String::new(),
        };
        links.push(Link {
            title,
            target: String::from(&link[1]),
        });
    }

    let note = Note {
        path: String::from(path),
        id: String::from(id),
        title: String::from(title),
        date: String::from(date),
        tasks,
        links,
        contents: String::from(contents)
    };
    Ok(note)
}

fn main() {
    let mut notes = HashMap::new();
    for file in std::fs::read_dir("./examples").unwrap() {
        let file = file.unwrap();
        let note = parse_org_file(file.path().to_str().unwrap()).expect("WRONG");
        notes.insert(String::from(note.id.to_string()), note);
    }

    println!("{:?}", notes);
}
