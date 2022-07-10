// ========================

// use anyhow::anyhow;
use fconx_scraper::config::Series;
use fconx_scraper::episode::Episode;
use futures::StreamExt;

///
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

///
#[tokio::main]
async fn main() -> Result<()> {
    //

    let vs = scrape(Series::OL).unwrap();
    for v in vs {
        println!("{:?}\n", v);
    }

    // ========================

    // test().await;

    // ========================

    Ok(())
}

///
fn scrape(series: Series) -> Result<Vec<Episode>> {
    let browser = {
        let opts = headless_chrome::LaunchOptionsBuilder::default()
            // .headless(false)
            .idle_browser_timeout(std::time::Duration::from_millis(10_000))
            .build()?;
        headless_chrome::Browser::new(opts)?
    };

    let tab = browser.wait_for_initial_tab()?;
    tab.navigate_to(&series.url().to_string())?;
    let elems = tab.wait_for_elements(".archive_entry").unwrap();

    let mut out = Vec::<Episode>::with_capacity(elems.len());

    let episode_num_sel = scraper::Selector::parse(".episode_number a").unwrap();
    let entry_content_sel = scraper::Selector::parse(".entry_content a").unwrap();
    let date_sel = scraper::Selector::parse(".date").unwrap();

    for elem in elems {
        let html = {
            // https://github.com/atroche/rust-headless-chrome/issues/73
            let remote_obj = elem
                .call_js_fn("function() { return this.innerHTML; }", true)
                .unwrap();
            let html_str = remote_obj.value.unwrap().to_owned().to_string();
            let html_str = html_str.replace("\\\"", "\"");
            scraper::Html::parse_fragment(html_str.as_str())
        };

        let number = {
            if let Some(elem_ref) = html.select(&episode_num_sel).next() {
                let number_raw = elem_ref.text().next().unwrap();
                println!("{}", number_raw);
                let number_str = number_raw.trim_start_matches("NO. ");
                if let Ok(number) = number_str.parse::<usize>() {
                    format!("{:04}", number)
                } else {
                    number_str.to_string()
                }
            } else {
                continue;
            }
        };

        let (title, page_url) = {
            if let Some(elem_ref) = html.select(&entry_content_sel).next() {
                let title = elem_ref.text().next().unwrap();
                let url = elem_ref.value().attr("href").unwrap();
                (title.to_string(), url.to_string())
            } else {
                continue;
            }
        };

        let date = {
            if let Some(elem_ref) = html.select(&date_sel).next() {
                let date_str = elem_ref.text().next().unwrap();
                chrono::NaiveDate::parse_from_str(&date_str, "%m/%d/%y").unwrap()
            } else {
                continue;
            }
        };

        // println!("{}", title);
        // println!("{}", number);
        // println!("");

        let episode = Episode::new(series, number, title, date, page_url);
        out.push(episode);
    }

    Ok(out)
}

// fn text_content(element: &headless_chrome::Element) -> Option<String> {
//     let remote_obj = element
//         .call_js_fn("function() { return this.textContent;}", true)
//         .unwrap();
//     remote_obj.value.map(|val| val.to_string())
// }
//
// fn get_attribute(element: &headless_chrome::Element, attribute: &str) -> Option<String> {
//     let attrs = element.get_attributes().unwrap().unwrap();
//     attrs.get(attribute).cloned()
// }
