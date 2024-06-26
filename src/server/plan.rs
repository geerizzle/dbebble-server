use std::{collections::BTreeMap, sync::Mutex};

use chrono::Local;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

use crate::{parser::plan::PlanParser, statics::API_URL};

use super::{
    cache::ServerCache, extract_date_time, extract_eva, generate_headers, generate_station_query,
};

pub struct PlanFetcher {
    client: Client,
    cache: Arc<Mutex<ServerCache>>,
}

impl PlanFetcher {
    pub fn new(cache: Arc<Mutex<ServerCache>>) -> Self {
        Self {
            client: Client::default(),
            cache,
        }
    }

    pub async fn start(&mut self) {
        let from = self.cache.lock().unwrap().get_start();
        let from_id = self.get_station_eva(&from).await.unwrap();
        let mut interval = time::interval(Duration::from_secs(3600));
        loop {
            if from_id == "quit" {
                println!("Qutting the program");
                break;
            }
            match self.get_current_plan(&from_id).await {
                Ok(times) => {
                    let mut cache = self.cache.lock().unwrap();
                    let destination = cache.get_destination();
                    println!("Next train to {:?}: {:?}", destination, times);
                    cache.update_cache(times);
                }
                Err(e) => {
                    println!("Error: {e:?}");
                }
            }
            interval.tick().await;
        }
    }

    pub async fn get_current_plan(
        &mut self,
        eva_id: &String,
    ) -> Result<BTreeMap<String, String>, String> {
        let plan_parser = PlanParser::new(Arc::clone(&self.cache));
        let time = Local::now().to_string();
        let (date, time) = extract_date_time(time);
        let url = format!("{}/plan/{}/{}/{}", API_URL, eva_id, date, time);
        let headers = generate_headers(&self.cache.lock().unwrap());
        let request = self.client.get(url).headers(headers);
        let response: String = request.send().await.unwrap().text().await.unwrap();
        let cache = self.cache.lock().unwrap();
        let destination = cache.get_destination().to_lowercase();
        let train_times = plan_parser.parse_plan(&response[..], &destination[..]);
        Ok(train_times)
    }

    pub async fn get_station_eva(&mut self, station: &String) -> Result<String, String> {
        let url = format!("{}/{}", API_URL, generate_station_query(station));
        let headers = generate_headers(&self.cache.lock().unwrap());
        let request = self.client.get(url).headers(headers);
        let response: String = request.send().await.unwrap().text().await.unwrap();
        let response = extract_eva(response);
        self.cache.lock().unwrap().update_eva_id(response.clone());
        Ok(response)
    }
}
