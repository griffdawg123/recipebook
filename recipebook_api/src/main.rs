mod llm_utils;
mod recipe_scraper;

use llm_utils::extract_recipe_info;
use recipe_scraper::scrape_webpage;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<recipe_scraper::ScraperError>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    let api_key =
        env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set in .env file");

    let url = "https://www.recipetineats.com/classic-lamingtons";

    println!("Scraping recipe from: {}", url);

    let page = match scrape_webpage(url).await {
        Ok(page) => {
            println!("✓ Successfully scraped: {}", page.title);
            page
        }
        Err(e) => {
            eprintln!("✗ Scraping error: {}", e);
            return Err(Box::new(e));
        }
    };

    println!("\nExtracting recipe information with LLM...");

    let recipe_info_result = extract_recipe_info(&page.content, &api_key).await;
    println!("Recipe Information:\n{:?}", recipe_info_result);

    Ok(())
}
