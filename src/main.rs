use parse::PageInfo;
use std::{collections::HashMap, env, path::PathBuf, time::Instant};
use ustr::UstrMap;

static OPENING_PAGE: &str = "<page>";
static CLOSING_PAGE: &str = "</page>";

mod parse;
mod tags;

use tags::{Tag, TagSet, TagsBuilder};

#[derive(Debug)]
enum MyError {
    Io(std::io::Error),
    InvalidPage(String),
}

impl From<std::io::Error> for MyError {
    fn from(err: std::io::Error) -> Self {
        MyError::Io(err)
    }
}

impl From<String> for MyError {
    fn from(err: String) -> Self {
        MyError::InvalidPage(err)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use parse::ParserRegexes;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let parser_regexes = ParserRegexes::default();
    let mut tag_counter = UstrMap::default();
    let mut pages = Vec::new();
    // Prints each argument on a separate line
    for file_to_parse in env::args().skip(1) {
        eprintln!("{}", file_to_parse);

        let file = File::open(file_to_parse)?;
        let total_bytes = file.metadata().unwrap().len();
        let buffer = BufReader::new(file);
        let mut page = String::new();
        let mut is_inside_page = false;

        let mut time_since_last_report = Instant::now();
        let mut total_bytes_seen = 0;
        let mut report_percentage_after = 0f64;

        for line in buffer.lines() {
            let line = line?;
            total_bytes_seen += line.len();

            if !is_inside_page && line.contains(OPENING_PAGE) {
                is_inside_page = true;
            } else {
                if line.contains(CLOSING_PAGE) {
                    parse::parse_page(&parser_regexes, &mut tag_counter, &mut pages, &page)?;

                    page.clear();

                    is_inside_page = false;
                } else {
                    page.push_str(&line);
                    page.push('\n');
                }
            }

            let percentage_seen = (total_bytes_seen as f64) / (total_bytes as f64);
            if percentage_seen > report_percentage_after {
                let current_instant = Instant::now();
                eprintln!(
                    "{}% complete in {:?}",
                    (report_percentage_after * 100f64).round(),
                    current_instant.duration_since(time_since_last_report.clone())
                );
                report_percentage_after += 0.05;
                time_since_last_report = current_instant;
            }
        }
    }

    eprintln!("{:#?}", pages.len());
    eprintln!("{:#?}", tag_counter);

    let fst_path = std::path::Path::new(&std::env::var_os("OUT_DIR").unwrap())
        .join("enwiktionary-word-tags.fst");
    let w = std::io::BufWriter::new(std::fs::File::create(fst_path)?);
    let mut tb = TagsBuilder::new(w)?;

    let mut pages_sorted = pages;
    pages_sorted.sort_by(|a, b| a.title.cmp(&b.title));

    for page in pages_sorted {
        tb.insert_tag_set(&page.title, &page.tags)?;

        println!("{}:{:?}", page.title, page.tags)
    }

    tb.finish()?;

    Ok(())
}

pub struct FSTOptions {
    /// Result will always be flattened to lowercase
    pub flatten_unicode: bool,
    pub exclude_pages_which_have_only_nouns: bool,
}

fn build_fst_from_pages(
    pages: &[PageInfo],
    options: FSTOptions,
    fst_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let w = std::io::BufWriter::new(std::fs::File::create(fst_path)?);
    let mut tb = TagsBuilder::new(w)?;

    let exclude_before_checking_empty = if options.exclude_pages_which_have_only_nouns {
        let mut excluded = TagSet::default();
        excluded.insert_tag(&Tag::Noun);
        excluded.insert_tag(&Tag::Pronoun);
        excluded.insert_tag(&Tag::ProperNoun);
        excluded
    } else {
        TagSet::default()
    };
    let mut pages_sorted = pages
        .iter()
        // flatten
        .filter_map(|info| {
            if info
                .tags
                .remove_tag_set(&exclude_before_checking_empty)
                .is_empty()
            {
                return None;
            }

            Some((
                if options.flatten_unicode {
                    unidecode::unidecode(&info.title).to_ascii_lowercase()
                } else {
                    info.title.to_lowercase()
                },
                info.tags.clone(),
            ))
        })
        .fold(
            HashMap::<String, TagSet>::new(),
            |mut acc, (title, tags)| {
                acc.entry(title).or_default().extend(tags);
                acc
            },
        )
        .into_iter()
        .collect::<Vec<_>>();
    pages_sorted.sort_by(|a, b| a.0.cmp(&b.0));

    for (title, tag_set) in pages_sorted {
        tb.insert_tag_set(&title, &tag_set)?;

        println!("{}:{:?}", title, tag_set)
    }

    tb.finish()?;

    Ok(())
}
/*
{
    u!("en-interj"): 2601,
    u!("en-proper noun"): 78508,
    u!("en-archaic second-person singular past of"): 333,
    u!("en-archaic third-person singular of"): 1682,
    u!("en-con"): 223,
    u!("en-simple past of"): 1310,
    u!("en-det"): 136,
    u!("en-symbol"): 158,
    u!("en-conj-simple"): 119,
    u!("en-pronoun"): 114,
    u!("en-plural noun"): 4598,
    u!("en-interjection"): 159,
    u!("en-irregular plural of"): 476,
    u!("en-third person singular of"): 132,
    u!("en-plural-noun"): 55,
    u!("en-third-person singular of"): 31455,
    u!("en-noun"): 411938,
    u!("en-pp"): 23,
    u!("en-PP"): 1597,
    u!("en-part"): 17,
    u!("en-decades"): 95,
    u!("en-suffix"): 954,
    u!("en-timeline"): 13600,
    u!("en-letter"): 60,
    u!("en-ing form of"): 68,
    u!("en-pron"): 435,
    u!("en-adverb"): 105,
    u!("en-number"): 38,
    u!("en-archaic second-person singular of"): 1892,
    u!("en-prop"): 1925,
    u!("en-prep"): 577,
    u!("en-note-upper case letter plural with apostrophe"): 2,
    u!("en-proper-noun"): 1607,
    u!("en-adjective"): 188,
    u!("en-conj"): 96,
    u!("en-particle"): 21,
    u!("en-verb"): 67296,
    u!("en-superlative of"): 2639,
    u!("en-proverb"): 589,
    u!("en-cont"): 431,
    u!("en-intj"): 399,
    u!("en-propn"): 32,
    u!("en-preposition"): 41,
    u!("en-phrase"): 2205,
    u!("en-conjunction"): 26,
    u!("en-adv"): 23988,
    u!("en-contraction"): 64,
    u!("en-prep phrase"): 323,
    u!("en-past of"): 33738,
    u!("en-adj"): 157591,
    u!("en-prefix"): 1569,
    u!("en-usage-equal"): 10,
    u!("en-comparative of"): 2430,
}
*/
