use anyhow::format_err;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use crate::{CLASSES_URL, get_html, PRACTICE_POSTFIX, ZSEM_URL};

#[derive(Debug, Serialize, Deserialize)]
pub struct Class {
    pub id: String,
    pub name: String,
    pub url: String,
    pub is_on_practice: bool,
}

pub async fn get_classes_list() -> anyhow::Result<Vec<Class>> {
    let mut classes = Vec::new();
    let not_found = "Element not found";

    let html = get_html(CLASSES_URL).await?;
    let document = Html::parse_document(&html);

    let table_selector = Selector::parse("table")
        .map_err(|e| format_err!(e.to_string()))?;

    let table = document.select(&table_selector)
        .next().ok_or(format_err!(not_found))?;

    let a_selector = Selector::parse("a[target=plan]")
        .map_err(|e| format_err!(e.to_string()))?;

    for element in table.select(&a_selector) {
        let href = element.value()
            .attr("href").ok_or(format_err!(not_found))?
            .to_string();
        let name = element.text().collect::<Vec<_>>().join(" ");

        // e.g. from /plany/o29.php to o29
        let id = href.split('/')
            .last().ok_or(format_err!(not_found))?
            .split('.').next().ok_or(format_err!(not_found))?
            .to_string();

        let is_on_practice = name.contains(PRACTICE_POSTFIX);

        let name = name.replace(PRACTICE_POSTFIX, "");
        let name = name.trim().to_string();

        classes.push(Class {
            id,
            name,
            url: format!("{}/{}", ZSEM_URL, href),
            is_on_practice,
        });
    }

    Ok(classes)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_classes_list() {
        let classes = get_classes_list().await.unwrap();
        assert!(!classes.is_empty());
    }
}