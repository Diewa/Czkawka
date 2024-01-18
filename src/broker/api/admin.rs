#![allow(renamed_and_removed_lints)]

use std::collections::HashMap;
use std::sync::Arc;

use rocket::State;

use rocket::response::content;
use rocket::form::{Form, FromForm};

use crate::topic::topic_service::{TopicService, TopicEntry, SubscriptionEntry};
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

            // TODO: better display the error in htmx - return 404 or something
            return content::RawHtml(format!("Can't create the topic due to {error}"));
        }
        Ok(_) => () // Ignore the Ok response
    }

    content::RawHtml(topic_entry.to_html())
}

#[get("/module/topic/<topic_name>")]
pub fn module_topic(topic_name: &str, topic_service: &State<Arc<TopicService>>, templater: &State<Arc<Templater>>) -> content::RawHtml<String> {

    let topic = match topic_service.get_topic(topic_name) {
        Ok(topic) => topic,
        Err(err) => {
            return content::RawHtml(format!("Can't find the topic {topic_name} due to {err}"));
        },
    };

    let vars = HashMap::from([
        ("name", topic.name.clone()),
        ("owner", topic.owner.clone()),
        ("subscribers", topic.subscribers.to_html())
    ]);

    content::RawHtml(templater.get("topic", vars))
}

#[get("/module/main")]
pub fn module_main(topic_service: &State<Arc<TopicService>>, templater: &State<Arc<Templater>>) -> content::RawHtml<String> {
    
    let topics = match topic_service.get_topics() {
        Err(error) => {
            return content::RawHtml(error.to_string());
        },
        Ok(topics) => topics,
    };

    // Construct main component
    let topics_vars = HashMap::from([
        ("topics", topics.0.to_html())
    ]);

    content::RawHtml(templater.get("main", topics_vars))
}

// Direct browser URL handlers
#[get("/")]
pub fn web_main(templater: &State<Arc<Templater>>) -> content::RawHtml<String>{
    
    // Construct index component
    let index_vars = HashMap::from([
        ("url", format!("/admin/module/main")),
        ("browser_url", format!("/admin")),
    ]);

    content::RawHtml(templater.get("index", index_vars))
}

#[get("/topic/<topic_name>")]
pub fn web_topic(topic_name: &str, templater: &State<Arc<Templater>>) -> content::RawHtml<String>{
    
    // Construct index component
    let index_vars: HashMap<&str, String> = HashMap::from([
        ("url", format!("/admin/module/topic/{topic_name}")),
        ("browser_url", format!("/admin/topic/{topic_name}")),
    ]);
    
    content::RawHtml(templater.get("main", index_vars))
}

#[derive(FromForm)]
pub struct SubscriberDTO {
    name: String,
    endpoint: String
}

#[post("/admin/topics/<topic_name>/subscribe", data = "<subscriber>")]
pub fn create_subscriber(
    topic_name: &str,
    subscriber: Form<SubscriberDTO>,
    topic_service: &State<Arc<TopicService>>, 
    // templater: &State<Arc<Templater>>
) {

    let subscription_entry = SubscriptionEntry {
        name: subscriber.name.clone(),
        endpoint: subscriber.endpoint.clone()
    };

    topic_service.subscribe_topic(topic_name, subscription_entry).unwrap();
    // templater.generateSth(subscriber)
    todo!()
}

trait ToHtml {
    fn to_html(&self) -> String;
}

impl<T: ToHtml> ToHtml for Vec<T> {
    fn to_html(&self) -> String {
        let mut html = String::new();
        for sub in self {
            html.push_str(&sub.to_html());
        }
        html
    }
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
                    <button hx-get=\"/admin/module/topic/{name}\" 
                            hx-target=\"#module\"
                            hx-push-url=\"/admin/topic/{name}\"
                            class=\"btn btn-secondary\">Edit
                    </button>
                </td>
            </tr>"))
    }
}

impl ToHtml for SubscriptionEntry {
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