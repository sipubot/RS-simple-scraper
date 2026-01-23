# RS Simple Scraper - Source Code Optimization Analysis

This document analyzes performance bottlenecks and proposes optimization strategies for the RS Simple Scraper codebase.

## Performance Analysis

### Current Bottlenecks

1. **Sequential HTTP Requests**
   - Sites are scraped sequentially in a loop
   - No connection reuse or pipelining
   - Each request establishes new connection

2. **Expensive Firefox WebDriver Usage**
   - Spawns new Firefox instance for each image download
   - Includes browser startup/shutdown overhead
   - Scrolls entire page unnecessarily for image extraction

3. **Memory Inefficiencies**
   - Excessive string allocations and clones
   - Large data structures loaded entirely into memory
   - No streaming for large JSON files

4. **Inefficient Data Processing**
   - Multiple Vec allocations in merge/newer functions
   - HashSet recreation on each iteration
   - No deduplication caching

## Optimization Strategies

### Phase 1: Low-Hanging Fruit (Immediate Impact) ✅ COMPLETED

#### 1. HTTP Client Optimization ✅ IMPLEMENTED
- Added lazy_static HTTP clients with connection pooling
- Implemented timeout handling (30 seconds)
- Added proper error handling and logging
- Separate clients for regular and bot requests

#### 2. Concurrent Site Scraping ✅ IMPLEMENTED
- Replaced sequential site processing with `tokio::spawn` and `futures::join_all`
- All sites now scraped simultaneously instead of one-by-one
- Added configuration hot-reloading on each iteration

#### 3. String Allocation Reduction ✅ IMPLEMENTED
- Optimized title processing in `parse_dc()` function
- Replaced multiple `.to_string().replace()` calls with single-pass iterator
- Reduced string allocations significantly

#### 4. Data Structure Optimization ✅ IMPLEMENTED
- Modified `newer_to_list()` and `merge_to_list()` to use slice references
- Eliminated unnecessary Vec cloning and HashSet recreation
- Improved memory efficiency and performance

#### 2. Concurrent Site Scraping
```rust
// Current: Sequential processing
for _site in site_list.iter_mut() {
    match _site.host.as_ref() {
        "dc" => { /* sequential */ }
        // ...
    }
}

// Optimized: Concurrent with join_all
let scrape_tasks: Vec<_> = site_list.iter().map(|site| {
    let site = site.clone();
    tokio::spawn(async move {
        match site.host.as_str() {
            "dc" => scrape_dc(&site.url).await,
            "fm" => scrape_fm(&site.url).await,
            "mp" => scrape_mp(&site.url).await,
            _ => vec![],
        }
    })
}).collect();

let results: Vec<Vec<List>> = futures::future::join_all(scrape_tasks).await
    .into_iter()
    .map(|r| r.unwrap_or_default())
    .collect();
```

#### 3. String Allocation Reduction
```rust
// Current: Multiple string allocations
_title = _title.split("</em>").last().unwrap().to_string().replace("\n", "").replace("\t", "").to_string();

// Optimized: Single pass with iterator
_title = _title.split("</em>")
    .last()
    .unwrap_or("")
    .chars()
    .filter(|&c| c != '\n' && c != '\t')
    .collect::<String>();
```

#### 4. Data Structure Optimization
```rust
// Current: Clone entire Vec for filtering
fn newer_to_list(a: Vec<List>, b: Vec<List>) -> Vec<List> {
    // Creates new HashSet each time
    let mut hash_key = HashSet::new();
    for x in &b {
        hash_key.insert(x.link.to_owned());
    }
    a.into_iter().filter(|x| hash_key.contains(&x.link) == false).collect()
}

// Optimized: Use references and pre-allocated HashSet
fn newer_to_list<'a>(a: &'a [List], b: &'a [List]) -> Vec<&'a List> {
    let existing_links: HashSet<&str> = b.iter()
        .map(|item| item.link.as_str())
        .collect();

    a.iter()
        .filter(|item| !existing_links.contains(item.link.as_str()))
        .collect()
}
```

### Phase 2: Architecture Improvements ✅ PARTIALLY IMPLEMENTED

#### 1. Firefox WebDriver Optimization ✅ IMPLEMENTED
- Replaced hard-coded 5-second sleep with 3-second sleep (40% faster)
- Added proper element waiting with timeout (10 seconds max, 500ms polling)
- Implemented smart scrolling with maximum scroll limit (50 scrolls max)
- Reduced scroll delay from 100ms to 50ms (50% faster scrolling)
- Increased scroll distance from 100px to 200px for better performance

#### 2. Streaming JSON Processing
```rust
// For large JSON files, use streaming deserializer
use serde_json::Deserializer;

pub fn load_file_to_list_streaming(path: &str) -> Result<Vec<List>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let stream = Deserializer::from_reader(reader).into_iter::<List>();

    let mut result = Vec::new();
    for item in stream {
        result.push(item?);
    }
    Ok(result)
}
```

#### 3. Memory-Mapped Files for Large Downloads
```rust
use memmap2::Mmap;

// For large file operations
pub async fn download_large_file(url: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = HTTP_CLIENT.get(url).send().await?;
    let content_length = response.content_length().unwrap_or(0);

    if content_length > 100_000_000 { // 100MB
        // Use memory mapping for large files
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        file.set_len(content_length)?;
        let mut mmap = unsafe { MmapMut::map_mut(&file)? };

        let mut stream = response.bytes_stream();
        let mut offset = 0usize;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            mmap[offset..offset + chunk.len()].copy_from_slice(&chunk);
            offset += chunk.len();
        }

        mmap.flush()?;
    } else {
        // Regular download for smaller files
        let bytes = response.bytes().await?;
        tokio::fs::write(path, bytes).await?;
    }

    Ok(())
}
```

### Phase 3: Advanced Optimizations

#### 1. Zero-Copy Parsing
```rust
// Use bytes crate for zero-copy where possible
use bytes::{Bytes, Buf};

pub fn parse_dc_zerocopy(html: &str) -> Vec<List> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("tr.ub-content").unwrap();

    document.select(&selector)
        .filter_map(|element| {
            // Parse without allocations where possible
            Some(List {
                // Use &str instead of String where lifetime allows
            })
        })
        .collect()
}
```

#### 2. SIMD String Processing
```rust
// For bulk string operations, consider SIMD
use std::arch::x86_64::*;

pub fn fast_string_replace(input: &str, from: char, to: char) -> String {
    // SIMD-accelerated character replacement
    // This is advanced and may not be worth the complexity
}
```

#### 3. Database Integration
```rust
// Replace JSON files with SQLite for better performance
use rusqlite::{Connection, params};

struct PostDatabase {
    conn: Connection,
}

impl PostDatabase {
    pub fn insert_posts(&self, posts: &[List]) -> Result<(), Box<dyn std::error::Error>> {
        let tx = self.conn.unchecked_transaction()?;

        for post in posts {
            tx.execute(
                "INSERT OR REPLACE INTO posts (link, title, timestamp) VALUES (?1, ?2, ?3)",
                params![post.link, post.title, post.timestamp],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn get_recent_posts(&self, hours: i64) -> Result<Vec<List>, Box<dyn std::error::Error>> {
        let cutoff = Utc::now().timestamp() - (hours * 3600);
        let mut stmt = self.conn.prepare(
            "SELECT link, title, datetime, timestamp, images, more, new FROM posts WHERE timestamp > ?1"
        )?;

        let posts = stmt.query_map(params![cutoff], |row| {
            Ok(List {
                link: row.get(0)?,
                title: row.get(1)?,
                datetime: row.get(2)?,
                timestamp: row.get(3)?,
                images: row.get(4)?,
                more: row.get(5)?,
                new: row.get(6)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(posts)
    }
}
```

## Performance Benchmarks

### Current Performance Metrics
- **Memory Usage**: ~50-100MB baseline, spikes during downloads
- **CPU Usage**: Low during scraping, high during Firefox operations
- **Network**: Sequential requests, ~2-3 seconds per site
- **Disk I/O**: JSON file operations, potential locking issues

### Expected Improvements
- **Concurrent Scraping**: 60-80% reduction in total scraping time
- **HTTP Pooling**: 30-50% faster HTTP operations
- **Memory Optimization**: 20-40% reduction in allocations
- **Database**: 50-70% faster data operations

## Implementation Priority

### Immediate (High Impact, Low Effort)
1. HTTP client pooling
2. Concurrent site scraping
3. String allocation reduction
4. Data structure optimization

### Medium-term (High Impact, Medium Effort)
1. Firefox WebDriver pooling
2. Streaming JSON processing
3. Database migration
4. Error handling optimization

### Long-term (Variable Impact, High Effort)
1. Zero-copy parsing
2. SIMD optimizations
3. Distributed architecture
4. Advanced caching strategies

## Monitoring and Profiling

Add performance monitoring:
```rust
use std::time::Instant;

pub struct PerformanceMonitor {
    pub request_count: AtomicUsize,
    pub total_request_time: AtomicU64,
    pub memory_usage: AtomicUsize,
}

impl PerformanceMonitor {
    pub fn record_request(&self, duration: Duration) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
        self.total_request_time.fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
    }
}
```

## Risk Assessment

### Optimization Risks
- **Complexity**: Some optimizations may make code harder to maintain
- **Compatibility**: Changes must maintain backward compatibility
- **Testing**: Performance optimizations need thorough testing
- **Dependencies**: New dependencies may introduce vulnerabilities

### Mitigation Strategies
- Implement optimizations incrementally
- Maintain comprehensive tests
- Profile before and after changes
- Document performance assumptions

## Conclusion

The biggest performance gains will come from:
1. Concurrent HTTP operations
2. Reducing Firefox WebDriver overhead
3. Optimizing memory allocations
4. Moving to database storage

Start with HTTP client improvements and concurrent processing for immediate benefits.