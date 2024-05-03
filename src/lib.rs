use anyhow::Result;
use reqwest::{header, Client};
use rss::{Channel, Item};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use tokio::{join, spawn};
pub mod consts;

pub type RSSMap<'a> = HashMap<Source, Cow<'a, [Article]>>;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    title: String,
    description: String,
    link: String,
    pub_date: String,
}

impl Article {
    pub fn title(&self) -> String {
        self.title.to_string()
    }
    pub fn description(&self) -> String {
        self.description.to_string()
    }
    pub fn link(&self) -> String {
        self.link.to_string()
    }
    pub fn pub_date(&self) -> String {
        self.pub_date.to_string()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Source {
    DevTo,
    MoKa,
    Mufiz,
}

impl From<Item> for Article {
    fn from(value: Item) -> Self {
        Self {
            title: value.title.unwrap_or_default(),
            description: value.description.unwrap_or_default(),
            link: value.link.unwrap_or_default(),
            pub_date: value.pub_date.unwrap_or_default(),
        }
    }
}

pub async fn get_channel(client: Client, url: &str) -> Result<Channel> {
    let resp = client.get(url).send().await?;
    let bytes = resp.bytes().await?;
    Ok(Channel::read_from(&bytes[0..])?)
}

pub async fn feed() -> Result<RSSMap<'static>> {
    let user_agent = header::HeaderValue::from_static("MK-RSS");
    let client = Client::builder()
        .default_headers({
            let mut headers = header::HeaderMap::new();
            headers.insert(header::USER_AGENT, user_agent);
            headers
        })
        .build()?;
    let devto_task = spawn(get_channel(client.clone(), consts::DEVTO));
    let moka_task = spawn(get_channel(client.clone(), consts::BLOG_MOKA));
    let mufiz_task = spawn(get_channel(client.clone(), consts::BLOG_MUFIZ));

    let mut joined_task = futures::future::join_all([devto_task, moka_task, mufiz_task]).await;

    let mufiz = joined_task.pop().unwrap()??;
    let moka = joined_task.pop().unwrap()??;
    let devto = joined_task.pop().unwrap()??;

    let mut devto_articles = Vec::new();
    // since we know all devto blogs are from me, we can do it quite easily
    for item in devto.items.clone() {
        devto_articles.push(Article::from(item))
    }

    let mut moka_articles = Vec::new();
    for item in moka.items.clone() {
        if item
            .dublin_core_ext()
            .clone()
            .is_some_and(|x| x.creators.contains(&"Mustafif Khan".to_string()))
        {
            moka_articles.push(Article::from(item))
        } else {
            continue;
        }
    }

    let mut mufiz_articles = Vec::new();
    for item in mufiz.items.clone() {
        mufiz_articles.push(Article::from(item))
    }

    let map = {
        let mut map = HashMap::new();
        _ = map.insert(Source::DevTo, Cow::from(devto_articles));
        _ = map.insert(Source::MoKa, Cow::from(moka_articles));
        _ = map.insert(Source::Mufiz, Cow::from(mufiz_articles));
        map
    };

    Ok(map)
}
