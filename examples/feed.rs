use mkblogs_rss::{feed, Source};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let feed = feed().await?;
    let devto = feed.get(&Source::DevTo).unwrap();
    let moka = feed.get(&Source::MoKa).unwrap();
    let mufiz = feed.get(&Source::Mufiz).unwrap();

    println!("Devto");
    for article in devto.iter(){
        println!("- {}", article.title())
    }

    println!("MoKa Reads");
    for article in moka.iter(){
        println!("- {}", article.title())
    }

    println!("Mufiz");
    for article in mufiz.iter(){
        println!("- {}", article.title())
    }

    Ok(())
}