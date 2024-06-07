mod basic;

use scraper::Html;
use tokio::{join, select};

use self::basic::BasicSelectorRankStep;

pub trait RankingStep {
    async fn run(dom: &Html) -> i32;
}

#[derive(Clone)]
pub struct RankingEngine {}
impl RankingEngine {
    pub async fn rank(dom: &Html) -> i32 {
        let (basic_sel,) = join! {
            BasicSelectorRankStep::run(dom),
        };

        basic_sel
    }
}
