use std::fmt::Debug;

pub fn print_vector<T: Debug>(title: &String, content: &Vec<T>, separator: char) {
    println!("{:?}", title);
    println!("[");
    for val in content {
        print!("{:?}{}", val, separator);
    }
    println!("]");
}