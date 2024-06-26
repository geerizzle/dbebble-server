use crate::server::cache::ServerCache;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use quick_xml::{events::Event, name::QName, Reader};

pub(crate) struct UpdateParser {
    cache: Arc<Mutex<ServerCache>>,
}

impl UpdateParser {
    pub fn new(cache: Arc<Mutex<ServerCache>>) -> Self {
        Self { cache }
    }

    pub(crate) fn parse_update(&self, response: &str) -> std::io::Result<()> {
        let mut reader = Reader::from_str(response);
        let mut change_detected = false;
        loop {
            match reader.read_event() {
                Ok(Event::Eof) => return Ok(()),
                Ok(Event::Start(ref e)) if e.name() == QName(b"s") => {
                    let id = e
                        .attributes()
                        .find(|a| a.as_ref().unwrap().key == QName(b"id"))
                        .unwrap();
                    let id_as_string =
                        String::from_utf8(id.unwrap().value.to_vec()).expect("Invalid UTF-8");

                    let cache = self.cache.lock().unwrap();
                    let current_plan: &BTreeMap<String, String> = cache.get_current_plan();
                    if current_plan.contains_key(&id_as_string) {
                        println!("Current Train {} has an update", cache.get_eva_id());
                        change_detected = true;
                    }
                }

                Ok(Event::Start(ref e)) if e.name() == QName(b"m") && change_detected => {
                    let update_type = e
                        .attributes()
                        .find(|a| a.as_ref().unwrap().key == QName(b"cat"));

                    if let Some(update_type) = update_type {
                        let update_type = String::from_utf8(update_type.unwrap().value.to_vec())
                            .expect("Invalid UTF-8");
                        println!("Update type: {}", update_type);
                    }
                }
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => {}
            }
        }
    }
}
