use icu_list::{ListFormatter, ListLength};
use icu_locid::LanguageIdentifier;
use icu_provider::DataLocale;
use icu_provider_adapters::fallback::LocaleFallbacker;
use icu_provider_blob::BlobDataProvider;
use writeable::Writeable;

const LIST_BLOB: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/i18n/data.postcard"));

thread_local! {
  static BLOB_PROVIDER: BlobDataProvider = BlobDataProvider::try_new_from_static_blob(LIST_BLOB).expect("should work");
  static FALLBACKER: LocaleFallbacker = BLOB_PROVIDER.with(|f| LocaleFallbacker::try_new_with_buffer_provider(f).unwrap());
}

pub fn format_list_and<W, I>(locale: LanguageIdentifier, values: I) -> String
where
  W: Writeable,
  I: Iterator<Item = W> + Clone,
{
  let formatter = FALLBACKER.with(|f| {
    let key_fallbacker = f.for_config(Default::default());
    let mut fallback_iterator = key_fallbacker.fallback_for(DataLocale::from(locale));

    BLOB_PROVIDER.with(|f| loop {
      let curr_step = fallback_iterator.get();
      let res =
        ListFormatter::try_new_and_with_length_with_buffer_provider(f, curr_step, ListLength::Wide);
      match res {
        Ok(out) => return out,
        _ => tracing::debug!("No fallback found for {}", curr_step),
      }
      fallback_iterator.step();
    })
  });

  formatter.format_to_string(values)
}
