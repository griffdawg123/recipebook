use reqwest;
use scraper::{Html, Selector};
use std::fmt;

#[derive(Debug)]
pub enum ScraperError {
    NetworkError(reqwest::Error),
    InvalidUrl(String),
    ParseError(String),
    TimeoutError,
    EmptyContent,
}

impl fmt::Display for ScraperError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScraperError::NetworkError(e) => write!(f, "Network error: {}", e),
            ScraperError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            ScraperError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ScraperError::TimeoutError => write!(f, "Request timeout"),
            ScraperError::EmptyContent => write!(f, "No content found"),
        }
    }
}

impl std::error::Error for ScraperError {}

impl From<reqwest::Error> for ScraperError {
    fn from(err: reqwest::Error) -> Self {
        ScraperError::NetworkError(err)
    }
}

#[derive(Debug)]
pub struct WebPage {
    pub url: String,
    pub title: String,
    pub content: String,
    pub html: String,
}

pub async fn scrape_webpage(url: &str) -> Result<WebPage, ScraperError> {
    if url.is_empty() {
        return Err(ScraperError::InvalidUrl("URL cannot be empty".to_string()));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| ScraperError::NetworkError(e))?;

    let response = client.get(url).send().await;

    match response {
        Ok(resp) => parse_response(resp, url).await,
        Err(err) if err.is_timeout() => {
            return Err(ScraperError::TimeoutError);
        }
        Err(err) => {
            return Err(ScraperError::NetworkError(err));
        }
    }
}

pub async fn parse_response(
    response: reqwest::Response,
    url: &str,
) -> Result<WebPage, ScraperError> {
    let html_content = response.text().await?;

    if html_content.trim().is_empty() {
        return Err(ScraperError::EmptyContent);
    }

    let document = Html::parse_document(&html_content);

    let title_selector =
        Selector::parse("title").map_err(|e| ScraperError::ParseError(e.to_string()))?;
    let title = document
        .select(&title_selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_else(|| "No title found".to_string());

    let body_selector =
        Selector::parse("body").map_err(|e| ScraperError::ParseError(e.to_string()))?;
    let content = document
        .select(&body_selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_else(|| html_content.clone());

    Ok(WebPage {
        url: url.to_string(),
        title,
        content,
        html: html_content,
    })
}

pub async fn get_webpage_content(url: &str) -> Result<String, ScraperError> {
    let page = scrape_webpage(url).await?;
    Ok(page.content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_url() {
        let result = scrape_webpage("").await;
        assert!(matches!(result, Err(ScraperError::InvalidUrl(_))));
    }

    #[tokio::test]
    async fn test_empty_url() {
        let result = get_webpage_content("").await;
        assert!(result.is_err());
    }
}
