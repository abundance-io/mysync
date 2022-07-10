use std::{fs::Metadata, clone};
use std::thread;
// use sysinfo::System;

use std::sync::mpsc;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug)]
struct Change {
    action: Action,
    directory: DirEntry,
}


#[derive(Clone)]
struct DirCache{
    Entry:DirEntry,
    metadata:Metadata
}

#[derive(Debug)]
enum Action {
    Create,
    Modify,
    Delete,
}

//compare two metaDataObjects
fn is_same(attr1: &Metadata, attr2: &Metadata) -> bool {
    let reqvec = vec![
        attr1.file_type() == attr2.file_type(),
        attr1.is_dir() == attr2.is_dir(),
        attr1.modified().unwrap() == attr2.modified().unwrap(),
        attr1.accessed().unwrap() == attr2.accessed().unwrap(),
        attr1.created().unwrap() == attr2.created().unwrap(),
        attr1.permissions() == attr2.permissions()
    ];
    
    // println!("{:?}", reqvec);
    return reqvec.into_iter().fold(true, |acc, x| acc && x);
}

pub fn watch(path: &str) {
    let mut prev_state = get_entries(path);
    let (tx, rx): (mpsc::Sender<Change>, mpsc::Receiver<Change>) = mpsc::channel();
    thread::spawn(move || {
        resp(rx);
    });
    loop {
        let new_state = get_entries(path);
        // println!("{:?}",new_state[0].metadata());
        // println!("{:?}",prev_state[0].metadata());
        for item in &new_state {
            let file_match = prev_state
                .clone()
                .into_iter()
                .find(|x| x.Entry.path() == item.Entry.path());
            match file_match {
                Some(file_match) => {
                    if !is_same(&file_match.metadata, &item.metadata) {
                        &prev_state.retain(|x| x.Entry.path() != item.Entry.path());
                        tx.send(Change {
                            action: Action::Modify,
                            directory: item.Entry.clone(),
                        })
                        .expect("send_err");
                    }
                }

                None => {
                    tx.send(Change {
                        action: Action::Create,
                        directory: item.Entry.clone(),
                    })
                    .expect("send_err");
                }
            }
        }

        //emit delete all unmatched items
        for item in &prev_state.clone() {
            if !new_state
                .clone()
                .into_iter()
                .any(|x| x.Entry.path() == item.Entry.path())
            {
                tx.send(Change {
                    action: Action::Delete,
                    directory: item.Entry.clone(),
                })
                .expect("send_err");

                &mut prev_state.retain(|x| x.Entry.path() != item.Entry.path());
            }
        }
        prev_state = new_state;
    }
}

fn resp(rx: mpsc::Receiver<Change>) {
    loop {
        let event = rx.recv().unwrap();
        println!("change occured {:?}", event);
    }
}

fn get_entries(path: &str) -> Vec<DirCache> {
    //get all files in directory
    let mut entries = vec![];
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        entries.push(
            DirCache{
               metadata:entry.metadata().unwrap(),
               Entry:entry
            }
            );
    }

    //println!("{:?}", entries);
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch() {
        watch("./test");
    }

    #[allow(dead_code)]
    fn test_entries() {
        get_entries("./test");
    }
}
