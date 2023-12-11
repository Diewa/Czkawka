#![allow(renamed_and_removed_lints)]

use rocket::State;

use rocket::response::content;
use rocket::form::{Form, FromForm};

use crate::topic::topic_service::{TopicService, TopicEntry};

#[derive(FromForm)]
pub struct TopicDTO {
    name: String,
    owner: String
}

#[post("/topics", data = "<new_topic>")]
pub fn create_topic(
    new_topic: Form<TopicDTO>,
    topic_service: &State<TopicService>
) -> content::RawHtml<String> {

    // TODO: Add validation
    let topic_entry = TopicEntry {
        name: new_topic.name.clone(),
        owner: new_topic.owner.clone()
    };

    // TODO: Good place to experiment with errors
    if topic_service.create_topic(topic_entry.clone()) {
        return content::RawHtml(topic_entry.to_html_row());
    }

    content::RawHtml(String::new())
}

// #[post("/topics2", data)]
// pub fn create_topic2() -> content::RawHtml<String> {
//     // TODO: Add validation
//     content::RawHtml(String::new())
// }

#[get("/topics")]
pub fn get_topics(topic_service: &State<TopicService>) -> content::RawHtml<String> {

    let mut html = String::new();

    for topic_entry in topic_service.get_topics() {
        html.push_str(&topic_entry.to_html_row());
    }
    
    content::RawHtml(html)
}

trait ToHtmlRow {
    fn to_html_row(&self) -> String;
}

impl ToHtmlRow for TopicEntry {
/*
    <tr>
        <td>Tomek-the-topic</td>
        <td>Stary Tomka</td>
    </tr>
*/
    fn to_html_row(&self) -> String {
        let mut html = String::new();
        html.push_str("<tr><td>");
        html.push_str(&self.name);
        html.push_str("</td><td>");
        html.push_str(&self.owner);
        html.push_str("</td></tr>");
        html
    }
}