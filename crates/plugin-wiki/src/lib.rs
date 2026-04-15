mod infobox;
mod options;
mod util;
mod wikilink;

pub use options::WikiOptions;

pub fn process(events: &[pendon_core::Event]) -> Vec<pendon_core::Event> {
    process_with_options(events, WikiOptions::default())
}

pub fn process_with_options(
    events: &[pendon_core::Event],
    options: WikiOptions,
) -> Vec<pendon_core::Event> {
    let infobox_processed = infobox::process_infobox(events, &options);
    wikilink::process_wikilinks(&infobox_processed, &options)
}
