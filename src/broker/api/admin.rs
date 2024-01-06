#![allow(renamed_and_removed_lints)]

use std::collections::HashMap;
use std::sync::Arc;

use rocket::State;

use rocket::response::content;
use rocket::form::{Form, FromForm};

use crate::topic::topic_service::{TopicService, TopicEntry, Subscriber};
use crate::api::templater::Templater;

#[derive(FromForm)]
pub struct TopicDTO {
    name: String,
    owner: String
}

#[post("/topics", data = "<new_topic>")]
pub fn create_topic(
    new_topic: Form<TopicDTO>,
    topic_service: &State<Arc<TopicService>>
) -> content::RawHtml<String> {

    // TODO: Add validation
    let topic_entry = TopicEntry {
        name: new_topic.name.clone(),
        owner: new_topic.owner.clone(),
        subscribers: vec![]
    };

    match topic_service.create_topic(topic_entry.clone()) {
        Err(error) => {
            // TODO: better display the error in html
            return content::RawHtml(error.to_string());
        }
        Ok(_) => () // Ignore the Ok response
    }

    content::RawHtml(topic_entry.to_html())
}

#[get("/topics")]
pub fn get_topics(topic_service: &State<Arc<TopicService>>) -> content::RawHtml<String> {

    let mut html = String::new();

    let topics = match topic_service.get_topics() {
        Err(error) => {
            return content::RawHtml(error.to_string());
        },
        Ok(topics) => topics,
    };

    for topic_entry in topics.iter() {
        html.push_str(&topic_entry.to_html());
    }
    
    content::RawHtml(html)
}

#[get("/topic/<name>")]
pub fn get_topic(name: &str, topic_service: &State<Arc<TopicService>>, templater: &State<Arc<Templater>>) -> content::RawHtml<String> {

    // Fetch all topics 
    let topics = match topic_service.get_topics() {
        Err(error) => {
            return content::RawHtml(format!("Error fetching topics! {}", error));
        },
        Ok(topics) => topics,
    };

    // Look for specific topic
    let mut topic_found = None;
    for topic in topics.iter() {
        if name == topic.name {
            topic_found = Some(topic);
        }
    }

    if !topic_found.is_some() {
        return content::RawHtml(format!("Topic {} doesn't exist!", name));
    }

    let topic = topic_found.unwrap();

    let vars = HashMap::from([
        ("name", topic.name.clone()),
        ("owner", topic.owner.clone()),
        ("subscribers", topic.subscribers.to_html())
    ]);

    templater.get("topic", vars)
}

#[get("/")]
pub fn index(templater: &State<Arc<Templater>>) -> content::RawHtml<String> {

    templater.get("main", HashMap::new())
}

trait ToHtml {
    fn to_html(&self) -> String;
}

impl ToHtml for TopicEntry {

    fn to_html(&self) -> String {
        let name = &self.name;
        let owner = &self.owner;
        String::from(&format!(
            "<tr class=\"align-middle\">
                <td>{name}</td>
                <td>{owner}</td>
                <td class=\"text-center\">
                    <button hx-get=\"/admin/topic/{name}\" 
                            hx-target=\"#module\" 
                            class=\"btn btn-secondary\">Edit
                    </button>
                </td>
            </tr>"))
    }
}

impl ToHtml for Subscriber {

    fn to_html(&self) -> String {
        let name = &self.name;
        let endpoint = &self.endpoint.to_string();
        String::from(format!(
            "<tr>
                <td>{name}</td>
                <td>{endpoint}</td>
            </tr>"
        ))
    }
}

impl ToHtml for Vec<Subscriber> {
    fn to_html(&self) -> String {
        let mut html = String::new();
        for sub in self {
            html.push_str(&sub.to_html());
        }
        html
    }
}