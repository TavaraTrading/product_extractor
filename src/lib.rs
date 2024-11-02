use serde_json::Value;
use scraper::{Html, Selector};
use pyo3::prelude::*;


#[pyfunction]
#[pyo3(signature = (html, category_selector=None))]
fn extract_product(html: &str, category_selector: Option<&str>) -> PyResult<Option<String>> {
    let document = Html::parse_document(html);

    // Extract JSON-LD Product data
    let mut json_ld = document
        .select(&Selector::parse("script[type=\"application/ld+json\"]").expect("Invalid selector"))
        .filter_map(|element| {
            let json_text = element.text().collect::<String>();
            serde_json::from_str::<Value>(&json_text).ok()
        })
        .find_map(|parsed| {
            // Check if the JSON-LD is of type Product
            match parsed.get("@type").and_then(Value::as_str) {
                Some("Product") => Some(parsed),
                _ => parsed.as_array()
                    .and_then(|arr| arr.iter().find(|item| item.get("@type") == Some(&Value::String("Product".to_string()))).cloned()),
            }
        });

    // Use the provided selector or default to "div#polku li:nth-child(n+3)"
    let selector = category_selector.unwrap_or("div#polku li:nth-child(n+3)");
    let category_selector = Selector::parse(selector).expect("Invalid selector");

    // Extract category hierarchy, split it into parts with whitespace and newlines stripped
    let category_parts: Vec<String> = document
        .select(&category_selector)
        .map(|element| {
            element.text()
                .collect::<String>()
                .replace('\n', "")     // Remove newlines
                .trim()                // Trim leading and trailing whitespace
                .to_string()           // Return a cleaned string
        })
        .collect();

    // Join category parts into a single, comma-separated string
    let category_str = category_parts.join(", ");

    // Add category data to JSON-LD if available
    if let Some(ref mut json_ld_data) = json_ld {
        // Add category parts as keywords
        json_ld_data["keywords"] = Value::Array(category_parts.iter().map(|part| Value::String(part.clone())).collect());

        // Add the joined category string as category
        json_ld_data["category"] = Value::String(category_str.clone());
    }

    // Convert JSON-LD to string for Python compatibility
    let json_ld_str = json_ld.map(|json| serde_json::to_string(&json).unwrap());

    Ok(json_ld_str)
}

/// A Python module implemented in Rust.
#[pymodule]
fn product_extractor(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Register the `extract_product` function with the module
    m.add_function(wrap_pyfunction!(extract_product, m)?)?;
    Ok(())
}
