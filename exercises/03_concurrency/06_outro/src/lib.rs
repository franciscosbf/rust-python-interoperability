use std::{sync::Arc, thread};

use dashmap::DashMap;
use pyo3::{exceptions::PyValueError, prelude::*, types::PySet};
use scraper::{Html, Selector};
use url::Url;

fn scrape(url: Url, visited: &DashMap<Url, bool>) -> Option<Vec<Url>> {
    let mut base = url.clone();
    base.set_query(None);
    base.set_fragment(None);

    if visited.contains_key(&base) {
        return None;
    }

    let res = ureq::get(url.as_str()).call().ok().or_else(|| {
        visited.insert(base.clone(), false);
        None
    })?;

    visited.insert(base.clone(), true);

    if res.content_type() != "text/html" {
        return None;
    }

    let body = res.into_string().ok().or_else(|| {
        visited.insert(base.clone(), false);
        None
    })?;

    base.set_path("");

    let fragment = Html::parse_fragment(&body);

    let hyperlinks = fragment
        .select(&Selector::parse("a").unwrap())
        .filter_map(|a| a.attr("href"))
        .chain(
            fragment
                .select(&Selector::parse("img").unwrap())
                .filter_map(|img| img.attr("src")),
        )
        .filter(|path| path.starts_with("/"))
        .filter_map(|path| {
            let hyperlink = base.join(path).ok()?;

            let mut hscraped = hyperlink.clone();
            hscraped.set_query(None);
            hscraped.set_fragment(None);

            (!visited.contains_key(&hscraped)).then_some(hyperlink)
        })
        .collect::<Vec<_>>();

    if hyperlinks.is_empty() {
        None
    } else {
        Some(hyperlinks)
    }
}

#[pyfunction]
/// Given a starting URL (`start_from`), discover all the URLs *on the same domain*
/// that can be reached by following links from the starting URL.
///
/// The discovered URLs should be inserted into the `site_map` set provided as an argument.
///
/// # Constraints
///
/// ## GIL
///
/// You should, as much as possible, avoid holding the GIL.
/// Try to scope the GIL to the smallest possible block of code—e.g. when touching `site_map`.
///
/// ## Threads
///
/// The program should use as many threads as there are available cores on the machine.
///
/// ## Invalid URLs
///
/// If a URL is invalid (e.g. it's malformed or it returns a 404 status code), ignore it.
///
/// ## External URLs
///
/// Do not follow links to external websites. Restrict your search to the domain of the
/// starting URL.
///
/// ## Anchors and Query Parameters
///
/// Ignore anchors and query parameters when comparing URLs.
/// E.g. `http://example.com` and `http://example.com#section` should be considered the same URL,
/// and normalizing them to `http://example.com` is the expected approach.
///
/// # Tooling
///
/// We recommend using the following crates to help you with this exercise:
///
/// - `ureq` for making HTTP requests (https://crates.io/crates/ureq)
/// - `scraper` for parsing HTML and extracting links (https://crates.io/crates/scraper)
/// - `url` for parsing URLs (https://crates.io/crates/url)
/// - `std`'s `sync` and `thread` modules for synchronization primitives.
///
/// Feel free to pull in any other crates you think might be useful.
/// If your approach is channel-based, you might want to use the `crossbeam` crate too.
pub fn site_map(py: Python<'_>, start_from: String, site_map: Bound<'_, PySet>) -> PyResult<()> {
    let visited = Arc::new(DashMap::new());

    py.allow_threads(|| {
        let start_from =
            Url::parse(&start_from).map_err(|_| PyValueError::new_err("invalid url"))?;

        let urls = if let Some(batch) = scrape(start_from, &visited) {
            batch
        } else {
            return Ok(());
        };

        let nw = thread::available_parallelism()
            .map(|nw| nw.get())
            .unwrap_or(1);

        let (distribute_sender, distribute_receiver) =
            crossbeam::channel::unbounded::<Option<Vec<Url>>>();

        let mut scrape_senders = vec![];
        let mut scrape_receivers = vec![];
        for _ in 0..nw {
            let (sender, receiver) = crossbeam::channel::unbounded::<Option<Url>>();
            scrape_senders.push(sender);
            scrape_receivers.push(receiver);
        }

        let dt = thread::spawn(move || {
            let mut batch = urls;
            let mut pending = batch.len();
            loop {
                let mut wi = 0;
                while let Some(url) = batch.pop() {
                    let _ = scrape_senders[wi].send(Some(url));
                    wi = (wi + 1) % nw;
                }

                loop {
                    let urls = distribute_receiver.recv().unwrap();
                    pending -= 1;
                    match urls {
                        Some(urls) => {
                            batch = urls;
                            pending += batch.len();
                            break;
                        }
                        None if pending == 0 => {
                            scrape_senders.iter().for_each(|w| w.send(None).unwrap());
                            return;
                        }
                        None => continue,
                    }
                }
            }
        });

        let mut ts = vec![dt];
        for scrape_receiver in scrape_receivers {
            let cvisited = visited.clone();
            let cdistribute_sender = distribute_sender.clone();

            let wt = thread::spawn(move || {
                while let Some(url) = scrape_receiver.recv().unwrap() {
                    cdistribute_sender.send(scrape(url, &cvisited)).unwrap();
                }
            });

            ts.push(wt);
        }

        for t in ts {
            let _ = t.join();
        }

        PyResult::Ok(())
    })?;

    for pair in visited.iter() {
        if *pair.value() {
            site_map.add(pair.key().to_string())?;
        }
    }

    Ok(())
}

#[pymodule]
fn outro3(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(site_map, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use pyo3::prepare_freethreaded_python;

    use super::*;

    #[test]
    fn test_site_map() {
        prepare_freethreaded_python();

        Python::with_gil(|py| {
            let set = PySet::empty(py).expect("failed to initialized set");

            site_map(py, "https://rust-exercises.com".to_string(), set.clone())
                .expect("site_map returned unexpected error");

            assert!(set.len() >= 200);
        });
    }
}
