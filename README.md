# RS Simple Scraper (TextMiner)

A simple Rust-based web scraper for Korean community websites. This tool periodically scrapes posts from DC Inside, Femco, and MLB Park, saves them to JSON files, and downloads images from specified posts.

## Features

- **Multi-site Scraping**: Supports DC Inside (dc), Femco (fm), and MLB Park (mp/mp_low)
- **Automatic Image Download**: Downloads images from DC Inside posts that match download criteria
- **JSON Storage**: Saves scraped posts to JSON files with timestamps
- **Continuous Monitoring**: Runs in a loop, checking for new posts every 5 minutes
- **Firefox Integration**: Uses Selenium WebDriver for JavaScript-heavy pages
- **Nick Filtering**: Filters out posts from specified users
- **Timezone Support**: Uses Seoul timezone for timestamps

## Project Structure

```
.
├── src/
│   ├── main.rs          # Main application logic
│   ├── utils.rs         # Utility functions for HTTP requests, file operations
│   └── foxfox.rs        # Firefox WebDriver integration
├── site.json            # Configuration for sites to scrape
├── save.json            # Configuration for save paths
├── down.json            # Configuration for download targets
├── nick.json            # Configuration for filtered nicknames
├── Cargo.toml
├── Cargo.lock
└── README.md
```

## Dependencies

- tokio: Asynchronous runtime
- reqwest: HTTP client
- scraper: HTML parsing
- serde: Serialization
- chrono: Date/time handling
- thirtyfour: Selenium WebDriver for Firefox
- regex: Regular expressions

## Setup

1. **Install Rust**: Make sure you have Rust installed. Download from [rustup.rs](https://rustup.rs/)

2. **Clone the repository**:
   ```bash
   git clone https://github.com/sipubot/RS-simple-scraper.git
   cd RS-simple-scraper
   ```

3. **Install dependencies**:
   ```bash
   cargo build --release
   ```

4. **Install GeckoDriver**: For Firefox automation, download GeckoDriver from [mozilla/geckodriver](https://github.com/mozilla/geckodriver/releases) and ensure it's in your PATH.

5. **Configure JSON files**:
   - `site.json`: List of sites to scrape with host and URL
   - `save.json`: Save paths for each host
   - `down.json`: Download targets (titles to match for image downloads)
   - `nick.json`: Nicknames to filter out

## Usage

1. **Run the scraper**:
   ```bash
   cargo run --release
   ```

2. The application will start monitoring the configured sites and save new posts to JSON files.

3. Images will be downloaded to the specified paths when matching posts are found.

## Configuration Files

### site.json
```json
[
    {
        "host": "dc",
        "url": "https://gall.dcinside.com/board/lists/?id=baseball_new12&exception_mode=recommend"
    }
]
```

### save.json
```json
[
    {
        "host": "dc",
        "json_path": "./data/dc_posts.json"
    }
]
```

### down.json
```json
[
    {
        "host": "dc",
        "title": "some_title",
        "path": "./downloads/dc/"
    }
]
```

### nick.json
```json
[
    {
        "nick": "filtered_user"
    }
]
```

## How It Works

1. Loads configuration from JSON files
2. Enters a loop that runs every 5 minutes
3. For each configured site:
   - Fetches HTML content
   - Parses posts using CSS selectors
   - Filters posts by time (last 48 hours) and nickname
   - Saves new posts to JSON
4. For DC Inside posts matching download criteria:
   - Uses Firefox WebDriver to load the page (handles dynamic content)
   - Extracts image URLs
   - Downloads images with proper referer headers

## Notes

- The scraper respects rate limits by running every 5 minutes
- Images are downloaded with referer headers to avoid 403 errors
- Posts older than 48 hours are automatically cleaned up
- Logging is saved to `./log/` directory with monthly rotation

## Development

- Built with Rust 2018 edition
- Uses async/await for concurrent operations
- Modular design with separate modules for scraping logic

## License

This project is private and for personal use.

## Author

SIPU <ddasik00@naver.com>