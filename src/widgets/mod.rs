use calculator::Calculator;
use reqwest::Client;
use serde::Serialize;

pub mod calculator;
pub mod color;
pub mod dice;
pub mod dictionary;
pub mod formula;
pub mod games;
pub mod joke;
pub mod metronome;
pub mod password;
pub mod quick_access;
pub mod time;
pub mod timer;
pub mod weather;
pub mod wikipedia;
pub mod xkcd;

use color::Color;
use dice::DiceRoll;
use dictionary::Dictionary;
use formula::Formula;
use games::Game;
use joke::Joke;
use metronome::Metronome;
use password::Password;
use quick_access::QuickAccess;
use time::Time;
use timer::Timer;
use weather::Weather;
use wikipedia::Wikipedia;
use xkcd::Xkcd;

#[derive(Debug, Serialize)]
pub enum Widget {
    Calculator(calculator::Calculator),
    Timer(Timer),
    Dictionary(Dictionary),
    Color(Color),
    DiceRoll(DiceRoll),
    Weather(Weather),
    Time(Time),
    Metronome(Metronome),
    Formula(Formula),
    Joke(Joke),
    Password(Password),
    Wikipedia(Wikipedia),
    Xkcd(Xkcd),
    QuickAccess(QuickAccess),
    Game(Game),
}

pub async fn detect_widget(query: &str, client: &Client, db: &sled::Db) -> Option<Widget> {
    if let Some(calculator) = Calculator::detect(query) {
        return Some(Widget::Calculator(calculator));
    }

    if let Some(quick_access) = QuickAccess::detect(query, client, db).await {
        return Some(Widget::QuickAccess(quick_access));
    }

    if let Some(timer) = Timer::detect(query) {
        return Some(Widget::Timer(timer));
    }

    if let Some(metronome) = Metronome::detect(query) {
        return Some(Widget::Metronome(metronome));
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

    if let Some(formula) = Formula::detect(query) {
        return Some(Widget::Formula(formula));
    }

    if let Some(joke) = Joke::detect(query, client).await {
        return Some(Widget::Joke(joke));
    }

    if let Some(passowrd) = Password::detect(query) {
        return Some(Widget::Password(passowrd));
    }

    if let Some(xkcd) = Xkcd::detect(query, client, db).await {
        return Some(Widget::Xkcd(xkcd));
    }

    if let Some(wikipedia) = Wikipedia::detect(query, client, db).await {
        return Some(Widget::Wikipedia(wikipedia));
    }

    if let Some(dictionary) = Dictionary::detect(query, client, db).await {
        return Some(Widget::Dictionary(dictionary));
    }

    if let Some(game) = Game::detect(query) {
        return Some(Widget::Game(game));
    }

    None
}
