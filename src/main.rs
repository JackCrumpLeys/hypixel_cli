#![feature(is_some_with)]
use crate::auction::{get_auctions, AuctionRoot, Auction};
use crate::util::print_middle;
use futures::future::ok;
use futures::{stream, StreamExt};
use rayon::prelude::*;
use reqwest::Client;
use std::io;
use std::io::{BufRead, Stdin, Write};
use mc_legacy_formatting::{Span, SpanExt};
use tabled::display::ExpandedDisplay;
use tabled::Table;

mod auction;
mod util;

#[tokio::main]
/// We make a request to the first page of auctions, then we make a request to every other page of
/// auctions, then we combine all the auctions into one big list
///
/// Returns:
///
/// A result with a possible error
async fn main() -> Result<(), reqwest::Error> {

    let timer = std::time::Instant::now();

    let mut auctions;

    match get_auctions().await {
        Ok(auctions_ok) => {
            auctions = auctions_ok;
        }
        Err(error) => {
            eprintln!("Got an error getting auctions: {}", error);
            panic!("could not fetch all auctions, check error and fix!")
        }
    }

    println!("==================================================");
    println!(
        "got {} actions in {} seconds",
        auctions.len(),
        timer.elapsed().as_secs_f32()
    );
    println!("==================================================\n");

    println!(
        "--------------------------------------------------------------------------------------"
    );
    print_middle(
        "Welcome to the Hypixel Skyblock Auction Checker",
        80,
    );
    print_middle("Made by smug rainbow#7777 on discord", 80);
    print_middle(
        "This program is open source and can be found at PLACEHOLDER",
        80,
    );
    print_middle(
        "This program is under the GNU General Public License v3.0",
        80,
    );
    print_middle(
        "You can view the license here: https://www.gnu.org/licenses/gpl-3.0.en.html",
        80,
    );
    print_middle(
        "If you have any questions or suggestions, please contact me on discord",
        80,
    );
    print_middle("Type help for a list of commands", 80);
    println!(
        "-------------------------------------------------------------------------------------"
    );

    let mut line: String = String::new();
    let stdin: Stdin = io::stdin();

    loop {
        if handle_input(&stdin, &mut line, &mut auctions).await.is_ok_and(|out| *out) {
            break;
        }
    }

    // for (i, auction) in auctions.iter().enumerate() {
    //     println!(
    //         "{} - {}. {}/{}",
    //         auction.tier,
    //         auction.item_name,
    //         i + 1,
    //         auctions.len()
    //     )
    // }

    Ok(())
}

async fn handle_input(stdin: &Stdin, line: &mut String, auctions: &mut Vec<Auction>) -> Result<bool, ()> {
    print!("=>");
    io::stdout().flush();

    stdin.lock().read_line(line).expect("Could not read line");

    let op = line.trim_end();
    if op == "help" {
        print_middle("======================================", 80);
        print_middle("help - Shows this help menu", 80);
        print_middle("exit - exit the application", 80);
        print_middle("update <auctions> - update data", 80);
        print_middle("get <item name> - gets all items on auction with that name", 80);
        print_middle("======================================", 80);
    } else if op.to_lowercase().starts_with("update ") {
        if op.to_lowercase().ends_with("auctions") {
            match get_auctions().await {
                Ok(auctions_ok) => {
                    *auctions = auctions_ok;
                    println!("updated data successfully!")
                }
                Err(error) => {
                    eprintln!("Got an error getting auctions: {}", error);
                    panic!("could not fetch all auctions, check error and fix!")
                }
            };
        }
    } else if op.to_lowercase().starts_with("get ") {
        let item = op.strip_prefix("get ").unwrap();
        let filtered = auctions.iter().filter(|auction| auction.item_name == item.to_string()).collect::<Vec<&Auction>>();
        println!("{}",  ExpandedDisplay::new(filtered))
    } else if op == "exit" {
        return Ok(true);
    } else {
        println!("could not find command {}", op);
    }
    line.clear();
    Ok(false)
}
