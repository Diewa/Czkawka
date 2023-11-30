use rocket::State;

use serde::Deserialize;
use rocket::serde::json::Json;

use crate::topic::topic_service::{TopicService, TopicEntry};

#[derive(Deserialize)]
pub struct TopicDTO {
    name: String,
    owner: String
}


/*
#[post("/articles", format = "json", data = "<new_article>")]
pub async fn post_articles(
    auth: Auth,
    new_article: Json<NewArticle>,
    db: Db,
)
*/

#[post("/topics")]
pub fn create_topic(
    new_topic: Json<TopicDTO>, 
    topicService: &State<TopicService>
) -> &'static str {

    let topic_entry = TopicEntry {
        name: new_topic.name,
        owner: new_topic.owner
    };

    topicService.create_topic(name.clone(), topic_entry);

    "OK!"
}

#[get("/topics")]
pub fn get_topics(topicService: &State<TopicService>) -> &'static str {
    "T1, T2, T3"
}

/*
// rustc --version
// rustup update





*/