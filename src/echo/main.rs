#[macro_use] extern crate rocket;

#[get("/echo/<id>/<text>")]
fn index(id: u64, text: String) -> String {
    format!("{} : {}", id, text)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}