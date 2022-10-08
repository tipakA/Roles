use icu::locid::{langid, LanguageIdentifier};
use icu_datagen::{CldrLocaleSubset, Out, SourceData};
use std::{fs::File, path::Path};

fn new_database() {
  let exists = Path::new("db.db").try_exists();
  if !matches!(exists, Ok(true)) {
    let _ = std::fs::copy("schema.db", "db.db");
  }
}

const DISCORD_LOCALES: &[LanguageIdentifier] = &[langid!("en-GB"), langid!("en"), langid!("pl")];

fn icu_gen_list() {
  icu_datagen::datagen(
    Some(DISCORD_LOCALES),
    &icu_datagen::keys(&[
      "list/and@1",
      "fallback/likelysubtags@1",
      "fallback/parents@1",
    ]),
    &SourceData::default()
      .with_cldr_for_tag("41.0.0", CldrLocaleSubset::Modern)
      .unwrap(),
    vec![Out::Blob(Box::new(
      File::create("i18n/data.postcard").unwrap(),
    ))],
  )
  .unwrap();
}

fn main() {
  new_database();
  icu_gen_list();
}
