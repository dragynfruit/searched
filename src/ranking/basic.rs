use scraper::Selector;

use super::RankingStep;

struct SelectorCriteria {
    selector: &'static str,
    rate: i32,
}

pub(super) struct BasicSelectorRankStep;
impl RankingStep for BasicSelectorRankStep {
    async fn run(dom: &scraper::Html) -> i32 {
        const SELECTORS: &'static [SelectorCriteria] = &[
            SelectorCriteria {
                selector:
                    "object, embed, marquee, blink, body[background], body[bgcolor], html[xmlns]",
                rate: 10,
            },
            SelectorCriteria {
                selector: "a[href$='.hqx'], a[href$='.aiff']",
                rate: 5,
            },
            SelectorCriteria {
                selector: "frame, table, img[src$='.gif'], img[src$='.GIF']",
                rate: 3,
            },
            SelectorCriteria {
                selector: "iframe, center",
                rate: 2,
            },
            SelectorCriteria {
                selector: "audio, nav, *[align]",
                rate: 1,
            },
            SelectorCriteria {
                selector: "img[src$='.webp'], img[src$='.jxl'], img[src$='.avif']",
                rate: -100,
            },
            SelectorCriteria {
                selector: "link[href$='.woff2'], link[href$='.woff']",
                rate: -10,
            },
            SelectorCriteria {
                selector: "img[src$='.svg'], html[itemtype]",
                rate: -5,
            },
            SelectorCriteria {
                selector:
                    "meta, script, link, img[src$='.png'], img[src$='.jpg'], img[src$='.jpeg']",
                rate: -1,
            },
        ];

        SELECTORS
            .iter()
            .map(|criteria| {
                dom.select(&Selector::parse(&criteria.selector).unwrap())
                    .count() as i32
                    * criteria.rate
            })
            .sum()
    }
}
