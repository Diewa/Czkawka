use rocket::State;

use serde::Deserialize;
use rocket::serde::json::Json;

use crate::topic::topic_service::{TopicService, TopicEntry};

#[derive(Deserialize)]
pub struct TopicDTO {
    name: String,
    owner: String
}

#[post("/topics", data = "<new_topic>")]
pub fn create_topic(
    new_topic: Json<TopicDTO>, 
    topic_service: &State<TopicService>
) -> &'static str {

    let topic_entry = TopicEntry {
        name: new_topic.0.name.clone(),
        owner: new_topic.0.owner
    };

    topic_service.create_topic(new_topic.0.name, topic_entry);

    "OK!"
}

#[get("/topics")]
pub fn get_topics(topic_service: &State<TopicService>) -> String {
    
    /*  
        <tr>
            <th scope="row">1</th> 
            <td>Mark</td>
            <td>Otto</td>
        </tr>
        <tr>
            <th scope="row">2</th>
            <td>Jacob</td>
            <td>Thornton</td>
        </tr>
        ...
    */

    let mut ret = String::new();

    for (index, topic) in topic_service.get_topics().iter().enumerate() {
        ret.push_str("<tr>");
        ret.push_str("<th scope=\"row\">");
        ret.push_str(&index.to_string());
        ret.push_str("</th><td>");
        ret.push_str(&topic.name);
        ret.push_str("</td><td>");
        ret.push_str(&topic.owner);
        ret.push_str("</td></tr>");
    }
    
    ret
}