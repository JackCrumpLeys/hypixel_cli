#![feature(is_some_with)]

use crate::auction::{get_auctions, Auction, AuctionRoot};
use crate::util::print_middle;
use futures::future::ok;
use futures::{stream, StreamExt};
use mc_legacy_formatting::{Span, SpanExt};
use nom::InputIter;
use quartz_nbt::{NbtCompound, NbtList, NbtReprError, NbtTag};
use rayon::prelude::*;
use reqwest::Client;
use std::collections::HashMap;
use std::env::args;
use std::io;
use std::io::{BufRead, Stdin, Write};
use std::ops::Index;
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
    print_middle("Welcome to the Hypixel Skyblock Auction Checker", 80);
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
        if handle_input(&stdin, &mut line, &mut auctions)
            .await
            .is_ok_and(|out| *out)
        {
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

async fn handle_input(
    stdin: &Stdin,
    line: &mut String,
    auctions: &mut Vec<Auction>,
) -> Result<bool, ()> {
    print!("=>");
    io::stdout().flush();

    stdin.lock().read_line(line).expect("Could not read line");

    let mut param_on = false;
    let op = line.trim();
    let flags = op
        .chars()
        .enumerate()
        .filter_map(|(i, char)| {
            if i == op.len() {
                return None;
            }
            if char == '-' {
                return Some(*op.chars().collect::<Vec<char>>().index(i + 1));
            }
            None
        })
        .collect::<Vec<char>>();
    let op = line
        .chars()
        .filter_map(|char| {
            if char == '-' {
                param_on = true;
                return None;
            }
            if param_on {
                param_on = false;
                return None;
            }
            Some(char.to_string())
        })
        .collect::<Vec<String>>()
        .join("");
    let op = op.trim();
    println!(
        "passed flags: {}",
        flags
            .iter()
            .map(|flag| format!("-{}", flag))
            .collect::<Vec<String>>()
            .join(", ")
    );

    if op == "help" {
        print_middle("======================================", 80);
        print_middle("help - Shows this help menu", 80);
        print_middle("exit - exit the application", 80);
        print_middle("update <auctions> - update data", 80);
        print_middle(
            "get [-b] <item name> - gets all items on auction with that name, -b for bin items",
            80,
        );
        print_middle("get_book [-b] <enchant name> [enchant level] - get a book with that enchantment on it and optionally that level", 80);
        print_middle("auctions best deals  <number of items to show> - Shows the best deals on bin items and gives you the command to buy it", 80);
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
        println!("getting {}", item);
        let mut filtered = auctions
            .iter()
            .filter(|auction| auction.item_name == item.to_string())
            .filter(|auction| {
                if flags.contains(&'b') {
                    return auction.bin;
                }
                true
            });
        println!(
            "{}",
            ExpandedDisplay::new(filtered.collect::<Vec<&Auction>>())
        );
    } else if op.to_lowercase().starts_with("get_book ") {
        let args = op
            .strip_prefix("get_book ")
            .unwrap()
            .split(" ")
            .collect::<Vec<&str>>();
        println!("arguments: {:?}", args);
        let enchant = *args.index(0);
        let level = args.get(1);
        println!("getting {}", enchant);
        let mut filtered = auctions
            .iter()
            .filter(|auction| {
                <NbtTag as TryInto<NbtCompound>>::try_into(
                    auction
                        .item_bytes
                        .nbt
                        .get::<_, &NbtList>("i")
                        .expect("invalid item nbt")[0]
                        .clone(),
                )
                .expect("invalid item nbt")
                .get::<_, i16>("id")
                .expect("invalid item nbt")
                    == 403
            })
            .filter(|auction| {
                let nbt = <NbtTag as TryInto<NbtCompound>>::try_into(
                    auction
                        .item_bytes
                        .nbt
                        .get::<_, &NbtList>("i")
                        .expect("invalid item nbt")[0]
                        .clone(),
                )
                .unwrap();

                let enchatments = match nbt
                    .get::<_, &NbtCompound>("tag")
                    .expect("invalid item nbt")
                    .get::<_, &NbtCompound>("ExtraAttributes")
                    .expect("invalid item nbt")
                    .get::<_, &NbtCompound>("enchantments")
                {
                    Ok(enchatments) => enchatments,
                    Err(_) => return false,
                };

                let has_enchantment = enchatments.contains_key(enchant);

                if has_enchantment && level.is_none() {
                    return true;
                } else if level.is_some() && has_enchantment {
                    let book_enchantment_level = enchatments.get::<_, i32>(enchant).unwrap();
                    return level
                        .unwrap()
                        .to_string()
                        .parse::<i32>()
                        .expect("invalid level number")
                        == book_enchantment_level;
                }
                return false;
            })
            .filter(|auction| {
                if flags.contains(&'b') {
                    return auction.bin;
                }
                true
            });
        println!(
            "{}",
            ExpandedDisplay::new(filtered.collect::<Vec<&Auction>>())
        );
    } else if op == "exit" {
        return Ok(true);
    } else {
        println!("could not find command {}", op);
    }
    line.clear();
    Ok(false)
}
