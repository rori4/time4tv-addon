#![feature(proc_macro_hygiene, decl_macro)]

// use lazy_static::*;
use chrono::Utc;
use rocket::*;
use rocket_contrib::json::Json;
use select::document::Document;
use select::predicate::Class;
use select::predicate::Name;
use std::error::Error;
use stremio_core::types::addons::*;
use stremio_core::types::*;

const TYPE_STR: &str = "channel";
const TIME4TV_BASE: &str = "http://www.time4tv.net/";
const INVALID_ID: &str = "time4tv-invalid-id";

// lazy_static! {
//     static ref GENRES: Vec<(String, String)> =
//         serde_json::from_str(include_str!("../genres_map.json")).unwrap();
// }

const MANIFEST_RAW: &str = include_str!("../manifest.json");

#[get("/manifest.json")]
//#[response(content_type = "json")]
fn manifest() -> String {
    MANIFEST_RAW.into()
}

#[get("/catalog/channel/time4tv.json")]
fn catalog() -> Option<Json<ResourceResponse>> {
    // @TODO error handling
    Some(Json(
        scrape_time4tv()
            .map(|metas| ResourceResponse::Metas { metas })
            // @TODO fix the unwrap
            .ok()?,
    ))
}

#[get("/meta/channel/<year>/<month>/<id>")]
fn meta_data(year: String, month: String, id: String) -> Option<Json<ResourceResponse>> {
    Some(Json(
        scrape_meta(format!("{}/{}/{}.php", year, month, id))
            .map(|metas_detailed| ResourceResponse::MetasDetailed { metas_detailed })
            .ok()?,
    ))
}

// #[get("/catalog/channel/time4tv/<genre>")]
// fn catalog_genre(genre: String) -> Option<Json<ResourceResponse>> {
//     // @TODO from name
//     let genre = GENRES.iter().find(|(id, _)| id == &genre)?;
//     Some(Json(
//         scrape_time4tv(&genre.0)
//             .map(|metas| ResourceResponse::Metas { metas })
//             // @TODO fix the unwrap
//             .ok()?,
//     ))
// }

fn scrape_meta(id: String) -> Result<Vec<MetaDetail>, Box<dyn Error>> {
    let url = format!("{}{}", TIME4TV_BASE, id);
    println!("{}", url);
    let resp = reqwest::get(&url)?;
    if !resp.status().is_success() {
        return Err("request was not a success".into());
    };

    Ok(Document::from_read(resp)?
        .find(Name("video"))
        .filter_map(|channel| {
            // if we cannot find name, we're probably finding the wrong items
            Some(MetaDetail {
                id: "Test".to_string(),
                poster: Some(get_poster_from_channel(&channel)?),
                name: get_name_from_channel(&channel)?,
                poster_shape: PosterShape::Square,
                type_name: "channel".into(),
                background: None,
                description: None,
                logo: None,
                popularity: 0.0,
                release_info: None,
                runtime: None,
                videos: [].to_vec(),
                featured_vid: None,
                external_urls: [].to_vec(),
            })
        })
        .collect())
}

fn scrape_time4tv() -> Result<Vec<MetaPreview>, Box<dyn Error>> {
    let url = format!("{}/categories", TIME4TV_BASE);
    let resp = reqwest::get(&url)?;
    if !resp.status().is_success() {
        return Err("request was not a success".into());
    };

    Ok(Document::from_read(resp)?
        .find(Name("li"))
        .filter_map(|channel| {
            // if we cannot find name, we're probably finding the wrong items
            let name = get_name_from_channel(&channel)?;
            Some(MetaPreview {
                id: get_id_from_channel(&channel).unwrap_or_else(|| INVALID_ID.to_owned()),
                type_name: TYPE_STR.to_owned(),
                poster: Some(get_poster_from_channel(&channel)?),
                name,
                poster_shape: PosterShape::Landscape,
            })
        })
        .collect())
}

fn get_id_from_channel(channel: &select::node::Node) -> Option<String> {
    channel
        .find(Name("a"))
        .next()?
        .attr("href")
        .map(|s| s.split('/').skip(3).collect::<Vec<&str>>().join("/"))
}

fn get_poster_from_channel(channel: &select::node::Node) -> Option<String> {
    let elem = channel.find(Name("img")).next()?;
    elem.attr("src")
        .or_else(|| elem.attr("data-original"))
        .map(|s| s.to_owned())
}

fn get_name_from_channel(channel: &select::node::Node) -> Option<String> {
    Some(
        channel
            .find(Class("channelName"))
            .next()?
            .text()
            .trim()
            .to_string(),
    )
}

fn get_streams_from_channel(destination: &str) -> Result<Vec<Video>, Box<dyn Error>> {
    let url = format!("{}", destination);
    let resp = reqwest::get(&url)?;
    if !resp.status().is_success() {
        return Err("request was not a success".into());
    };

    Ok(Document::from_read(resp)?
        .find(Name("video"))
        .filter_map(|channel| {
            // if we cannot find name, we're probably finding the wrong items
            // let name = get_name_from_channel(&channel)?;
            Some(Video {
                id: get_id_from_channel(&channel).unwrap_or_else(|| INVALID_ID.to_owned()),
                title: "test".to_string(),
                released: Utc::now(),
                overview: Some("test".to_string()),
                thumbnail: Some("test".to_string()),
                streams: None,
                series_info: None,
            })
        })
        .collect())
}

fn main() {
    let cors = rocket_cors::CorsOptions::default().to_cors().unwrap();

    rocket::ignite()
        .mount("/", routes![manifest, catalog, meta_data])
        .attach(cors)
        .launch();
}
