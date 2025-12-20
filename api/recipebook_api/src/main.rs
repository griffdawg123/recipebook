// Async entry point enabled by Tokio
// Allows us to use `.await` inside `main`
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Send a simple GET request to the target URL
    let body = reqwest::get("https://example.com")
        .await? // wait for the HTTP response
        .text() // read response body as text
        .await?; // wait for the full body to be collected

    // Print the raw HTML response
    println!("{}", body);

    Ok(())
}
