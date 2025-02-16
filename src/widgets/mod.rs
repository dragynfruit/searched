use reqwest::Client;
use serde::Serialize;

pub mod color;
pub mod dice;
pub mod dictionary;
pub mod time;
pub mod timer;
pub mod unit_converter;
pub mod weather;

use color::Color;
use dice::DiceRoll;
use dictionary::Dictionary;
use time::Time;
use timer::Timer;
use unit_converter::UnitConverter;
use weather::Weather;

#[derive(Debug, Serialize)]
pub enum Widget {
    UnitConverter(UnitConverter),
    Timer(Timer),
    Dictionary(Dictionary),
    Color(Color),
    DiceRoll(DiceRoll),
    Weather(Weather),
    Time(Time),
}

pub async fn detect_widget(query: &str, client: &Client, db: &sled::Db) -> Option<Widget> {
    if let Some(converter) = UnitConverter::detect(query) {
        return Some(Widget::UnitConverter(converter));
    }

    if let Some(timer) = Timer::detect(query) {
        return Some(Widget::Timer(timer));
    }

    if let Some(dictionary) = Dictionary::detect(query, client).await {
        return Some(Widget::Dictionary(dictionary));
    }

    if let Some(color) = Color::detect(query) {
        return Some(Widget::Color(color));
    }

    if let Some(dice) = DiceRoll::detect(query) {
        return Some(Widget::DiceRoll(dice));
    }

    if let Some(weather) = Weather::detect(query, client, db).await {
        return Some(Widget::Weather(weather));
    }

    if let Some(time) = Time::detect(query) {
        return Some(Widget::Time(time));
    }

    None
}
