use std::io::{Cursor, Read};
use base64::decode;
use fastnbt::error::Error;
use futures::{stream, StreamExt};
use reqwest::Client;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use quartz_nbt::{io, io::Flavor, NbtCompound, serde::{serialize, deserialize}};
use serde_json::Value;
use tabled::Tabled;
use mc_legacy_formatting::{SpanExt, Span, Color, Styles};
use fastnbt::from_bytes;
use flate2::read::GzDecoder;
use stringreader::StringReader;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Tabled)]
#[serde(rename_all = "camelCase")]
/// `AuctionRoot` is a struct that contains a boolean, two integers, a timestamp, and a vector of
/// `Auction`s.
///
/// The `Auction` type is a bit more complicated, but it's still pretty simple.
///
/// Properties:
///
/// * `success`: Whether or not the request was successful.
/// * `page`: The current page of auctions.
/// * `total_pages`: The total number of pages of auctions.
/// * `total_auctions`: The total number of auctions that exist for the given realm.
/// * `last_updated`: The time in milliseconds since the epoch when the auction data was last updated.
/// * `auctions`: A list of auctions.
pub struct AuctionRoot {
    pub success: bool,
    pub page: i64,
    pub total_pages: i64,
    pub total_auctions: i64,
    pub last_updated: i64,
    #[tabled(skip)]
    pub auctions: Vec<Auction>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Tabled)]
#[serde(rename_all = "camelCase")]
/// Properties:
///
/// * `uuid`: The unique ID of the auction.
/// * `auctioneer`: The name of the player who created the auction.
/// * `profile_id`: The UUID of the player who created the auction.
/// * `coop`: the players who are in the profiles coop
/// * `start`: The time the auction started
/// * `end`: The end time of the auction in epoch milliseconds.
/// * `item_name`: The name of the item
/// * `item_lore`: The lore of the item
/// * `extra`: This is a string that can be used to store extra information about the auction.
/// * `category`: The category of the item.
/// * `tier`: The tier of the item.
/// * `starting_bid`: The starting bid for the auction.
/// * `item_bytes`: The item bytes of the item being auctioned.
/// * `claimed`: Whether or not the auction has been claimed by the auctioneer.
/// * `claimed_bidders`: A list of bidders who have claimed the item.
/// * `highest_bid_amount`: The highest bid amount for this auction.
/// * `last_updated`: The last time the auction was updated.
/// * `bin`: Whether or not the auction is a bin auction.
/// * `bids`: A list of bids on the auction.
/// * `item_uuid`: The UUID of the item.
pub struct Auction {
    pub uuid: String,
    #[tabled(skip)]
    pub auctioneer: String,
    #[serde(rename = "profile_id")]
    #[tabled(skip)]
    pub profile_id: String,
    #[tabled(skip)]
    pub coop: Vec<String>,
    #[tabled(skip)]
    pub start: i64,
    #[tabled(skip)]
    pub end: i64,
    #[serde(rename = "item_name")]
    pub item_name: String,
    #[serde(rename = "item_lore")]
    #[tabled(display_with = "display_minecraft_text")]
    pub item_lore: String,
    #[tabled(skip)]
    pub extra: String,
    pub category: String,
    pub tier: String,
    #[serde(rename = "starting_bid")]
    pub starting_bid: i64,
    #[serde(rename = "item_bytes")]
    #[tabled(skip)]
    pub item_bytes: NbtMetadata,
    #[tabled(skip)]
    pub claimed: bool,
    #[serde(rename = "claimed_bidders")]
    #[tabled(skip)]
    pub claimed_bidders: Vec<Value>,
    #[serde(rename = "highest_bid_amount")]
    pub highest_bid_amount: i64,
    #[serde(rename = "last_updated")]
    pub last_updated: i64,
    pub bin: bool,
    #[tabled(skip)]
    pub bids: Vec<Bid>,
    #[serde(rename = "item_uuid")]
    #[tabled(skip)]
    pub item_uuid: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(from = "String")]
pub struct NbtMetadata {
    pub nbt: NbtCompound,
    pub root: String,
}

impl From<String> for NbtMetadata {
    fn from(s: String) -> Self {
        let s = &decode(s).unwrap()[..];
        let mut decoder = GzDecoder::new(s);
        let mut data = vec![];
        decoder.read_to_end(&mut data).unwrap();
        let (nbt, root) = io::read_nbt(&mut Cursor::new(data), Flavor::Uncompressed).unwrap();
        // println!("{:?}", nbt);
        Self {
            nbt,
            root
        }

    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    #[serde(rename = "Unbreakable")]
    pub unbreakable: i64,
    #[serde(rename = "HideFlags")]
    pub hide_flags: i64,
    pub display: Display,
    #[serde(rename = "ExtraAttributes")]
    pub extra_attributes: ExtraAttributes,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Display {
    #[serde(rename = "Lore")]
    pub lore: Vec<String>,
    #[serde(rename = "Name")]
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraAttributes {
    pub id: String,
    pub uuid: String,
    pub timestamp: String,
}



fn display_minecraft_text(text: &String) -> String {
    text.span_iter().map(Span::wrap_colored).map(|span| format!("{}", span)).collect::<Vec<String>>().join("")
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Tabled)]
#[serde(rename_all = "camelCase")]
/// `Bid` is a struct with 5 fields, `auction_id`, `bidder`, `profile_id`, `amount`, and `timestamp`.
///
///
/// Properties:
///
/// * `auction_id`: The ID of the auction that the bid is for.
/// * `bidder`: The bidder's name
/// * `profile_id`: The ID of the bidder's profile.
/// * `amount`: The amount of the bid.
/// * `timestamp`: The time the bid was made.
pub struct Bid {
    #[serde(rename = "auction_id")]
    pub auction_id: String,
    pub bidder: String,
    #[serde(rename = "profile_id")]
    pub profile_id: String,
    pub amount: i64,
    pub timestamp: i64,
}

pub async fn get_auctions() -> Result<Vec<Auction>, reqwest::Error> {
    let auction_first_response = reqwest::get("https://api.hypixel.net/skyblock/auctions?https://api.hypixel.net/skyblock/auctions?page=0").await?.json::<AuctionRoot>().await?;
    let mut auctions = auction_first_response.auctions.clone();
    let pages: i32 = auction_first_response.total_pages.clone() as i32;

    // (1..pages).par_iter().for_each(|page| auctions.append(&mut reqwest::blocking::get(format!("https://api.hypixel.net/skyblock/auctions?https://api.hypixel.net/skyblock/auctions?page={}", page))?.json::<AuctionRoot>()?.auctions.clone()));

    // for page in (1..pages) {
    //     auctions.append(&mut reqwest::blocking::get(f!("https://api.hypixel.net/skyblock/auctions?https://api.hypixel.net/skyblock/auctions?page={}", page))?.json::<AuctionRoot>()?.auctions.clone())
    // }

    let client = Client::new();

    let urls = (1..pages).into_iter().map(|page| format!("https://api.hypixel.net/skyblock/auctions?https://api.hypixel.net/skyblock/auctions?page={}", page)).collect::<Vec<String>>();

    let bodies = stream::iter(urls)
        .enumerate()
        .map(|(i, url)| {
            let client = &client;
            async move {
                let resp = client.get(url).send().await?;
                resp.json::<AuctionRoot>().await
            }
        })
        .buffer_unordered(pages as usize);

    let auction_pages = bodies.enumerate().then(|(i, auction)| async move {
        match auction {
            Ok(auction) => {
                println!(
                    "got page: {}/{}, with {} auctions",
                    i,
                    auction.total_pages,
                    auction.auctions.len()
                );
                auction
            }
            Err(error) => {
                eprintln!("Got an error getting page: {}", error);
                panic!("could not fetch all auctions, check error and fix!")
            }
        }
    });
    println!("--------------------------------------------------------------------------------------");
    for auction in auction_pages.collect::<Vec<AuctionRoot>>().await {
        auctions.extend(auction.auctions);
    }
    println!("--------------------------------------------------------------------------------------\n");

    Ok(auctions)
}
