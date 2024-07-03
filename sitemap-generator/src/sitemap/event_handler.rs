use rayon::prelude::*;
use std::{
    fs::{read_dir, File},
    io::BufReader,
};

use crate::queries::event_subjects_updated::Event;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};
use tracing::warn;

use super::UrlSet;

pub struct EventHandler {
    receiver: Receiver<Event>,
}

impl EventHandler {
    pub fn start(receiver: Receiver<Event>) -> JoinHandle<()> {
        let mut s = Self { receiver };
        tokio::spawn(s.listen())
    }

    async fn listen(mut self) {
        while let Some(message) = self.receiver.recv().await {
            match message {
                Event::ProductCreated(product) => {}
                Event::ProductUpdated(product) => {}
                Event::ProductDeleted(product) => {}
                Event::CategoryCreated(category) => {}
                Event::CategoryUpdated(category) => {}
                Event::CategoryDeleted(category) => {}
                Event::CollectionCreated(collection) => {}
                Event::CollectionUpdated(collection) => {}
                Event::CollectionDeleted(collection) => {}
                Event::PageCreated(page) => {}
                Event::PageUpdated(page) => {}
                Event::PageDeleted(page) => {}
                Event::Unknown => warn!("Unknown event called"),
            }
        }
    }
}

async fn read_xmls() {
    let paths = read_dir(std::env::var("SITEMAP_TARGET_FOLDER").unwrap()).unwrap();
    let mut all_urls: Vec<UrlSet> = paths
        .into_iter()
        .par_bridge()
        .filter_map(|path| {
            if let Ok(path) = path {
                if path.path().is_file() {
                    let file = File::open(path.path()).expect("Unable to open file");
                    let reader = BufReader::new(file);
                    return Some(quick_xml::de::from_reader(reader).unwrap());
                }
            }
            return None;
        })
        .collect();
}
