use std::{fs::OpenOptions, io::Read, collections::HashMap};

use rocket::response::content;


pub struct Templater {
    web_path: String
}

struct Variable {
    name: String,
    start_index: usize,
    end_index: usize
}

impl Templater {
    pub fn new(web_path: &str) -> Self {
        Templater { web_path: web_path.to_owned() }
    }

    pub fn get(&self, component_name: &str, variables: HashMap<&str, String>) -> content::RawHtml<String> {

        // Construct name of the component file
        let file_name = self.web_path.clone() + "/" + component_name + ".component";
        
        // Load file. TODO: cache static files upon request?
        let mut file = OpenOptions::new()
            .read(true)
            .open(&file_name)
            .expect(&("Failed to open file".to_owned() + &file_name));

        // Read file into the buffer
        let mut buffer = vec![];
        file.read_to_end(&mut buffer).unwrap();

        // Assemble the component from file content and variables
        let mut result = String::new();

        let mut buffer_index = 0;
        for var in Templater::parse(&buffer) {

            // 1. Insert text from the last index till start of the next variable
            result.push_str(std::str::from_utf8(&buffer[buffer_index .. var.start_index]).unwrap());

            // 2. Add the variable itself
            match variables.get(var.name.as_str()) {
                None => {
                    return content::RawHtml(
                        format!("Error parsing template! Missing attribute: {}", var.name));
                }
                Some(html) => {
                    result.push_str(html);
                }
            }

            // 3. Current index sits at the end of the variable
            buffer_index = var.end_index;
        }

        // 4. Add the remaining text
        result.push_str(std::str::from_utf8(&buffer[buffer_index..]).unwrap());
        return content::RawHtml(result);
    }

    fn parse(html: &Vec<u8>) -> Vec<Variable> {

        let mut res = vec![];
        let mut is_reading = false;
        let mut index = 0;

        for (i, letter) in html.iter().enumerate() {
            if letter == &b'{' {
                is_reading = true;
                index = i;
            }
            else if letter == &b'}' {
                if is_reading && i - index > 1 {
                    res.push(Variable { 
                        name: String::from_utf8(html[index + 1 .. i].to_vec()).unwrap().trim().to_string(),
                        start_index: index,
                        end_index: i + 1
                    });
                }
                is_reading = false;
            }
        }

        res
    }
}

#[test]
fn parse_test() {
    let str = String::from("Some text\nmore text\n <b> {my_variable} </b> \neven more text").bytes().collect();
    let var = &Templater::parse(&str)[0];

    assert_eq!(var.name, String::from("my_variable"));
    assert_eq!(var.start_index, 25);
    assert_eq!(var.end_index, 38);
}

#[test]
fn parse_space_test() {
    let str = String::from("asd { ddd } dsa").bytes().collect();
    let var = &Templater::parse(&str)[0];

    assert_eq!(var.name, String::from("ddd"));
    assert_eq!(var.start_index, 4);
    assert_eq!(var.end_index, 11);
}