#![feature(int_roundings)]

use std::thread::current;

pub fn print_middle(msg:&str, width:i32) {

    let words = msg.split_whitespace();

    let mut lines = vec!["".to_string()];
    let mut current_line_index = 0;
    let mut current_line_length = 0;
    for word in words {
        let chars = word.len();
        if current_line_length+chars > width as usize {
            lines.push("".parse().unwrap());
            current_line_index += 1;
        }
        lines[current_line_index] += &*(word.to_owned() + " ");
        current_line_length += chars + 1
    }

    lines = lines.iter().map(|str| str.trim().to_string()).collect::<Vec<String>>();

    for line in lines {
        let effective_width = width - line.len() as i32;

        let padding = effective_width / 2;
        let literal_padding = " ".repeat(padding as usize);
        println!("{}{}{}", literal_padding, line, literal_padding)
    }
}

#[test]
fn test_print_middle() {
    print_middle("======================================", 80);
    print_middle("help - Shows this help menu", 80);
    print_middle("exit - exit the application", 80);
    print_middle("update <auctions> - update data", 80);
    print_middle("get <item name> [-r] - gets all items on auction with that name with -r flag for regex", 80);
    print_middle("======================================", 80);

}