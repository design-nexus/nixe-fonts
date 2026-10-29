use crate::widgets::{self, Page};
use crate::{catalog, events, fmt, paths, prefs, window};
use gtk::glib;
use gtk::prelude::*;

pub fn build(page: &Page) {
    let p = prefs::get();

    // ----- Installing -----
    let g = page.group("Installing");
    g.note(&format!(
        "Fonts install for your user only, in <tt>{}</tt>.",
        glib::markup_escape_text(&paths::pretty(&paths::fonts_dir()))
    ));
    g.add(&widgets::segmented_row(
        "Google Fonts files",
        "Static files have one file per style and work everywhere. Variable files hold every weight in one.",
        widgets::opts(&[("static", "Static"), ("variable", "Variable"), ("both", "Both")]),
        &p.google_files,
        |v| prefs::update(|p| p.google_files = v),
    ));
    g.add(&widgets::segmented_row(
        "Nerd Fonts variants",
        "Mono has single-width icons, for terminals. The regular and Propo variants suit editors and documents.",
        widgets::opts(&[("all", "All"), ("mono", "Mono only"), ("regular", "No Mono")]),
        &p.nerd_variants,
        |v| prefs::update(|p| p.nerd_variants = v),
    ));
    let (r, _) = widgets::switch_row(
        "Refresh the font cache",
        "Run fc-cache after installing or removing, so other apps see the change straight away.",
        p.refresh_cache,
        |on| prefs::update(|p| p.refresh_cache = on),
    );
    g.add(&r);

    // ----- Preview -----
    let g = page.group("Preview");
    let sample = gtk::Entry::new();
    sample.set_text(&p.sample);
    sample.set_width_chars(18);
    sample.connect_changed(|e| {
        prefs::update(|p| p.sample = e.text().to_string());
        events::emit(events::Change::Preferences);
    });
    g.add(&widgets::row("Preview text", "Sample shown when previewing a font.", Some(sample.upcast_ref())));
    let size = gtk::SpinButton::with_range(8.0, 96.0, 1.0);
    size.set_value(p.preview_size);
    size.connect_value_changed(|s| { prefs::update(|p| p.preview_size = s.value()); events::emit(events::Change::Preferences); });
    g.add(&widgets::row("Preview size", "Text size in points.", Some(size.upcast_ref())));
    let (r, _) = widgets::button_row("Clear the preview cache", "Delete downloaded preview files.", "Clear", |_| {
        let dir = paths::preview_dir();
        let size: u64 =
            std::fs::read_dir(&dir).map(|d| d.flatten().filter_map(|e| e.metadata().ok()).map(|m| m.len()).sum()).unwrap_or(0);
        let _ = std::fs::remove_dir_all(&dir);
        window::toast(&format!("Cleared {} of previews.", fmt::bytes(size)));
    });
    g.add(&r);

    // ----- Catalog -----
    let g = page.group("Catalog");
    let (row, _) = widgets::choice_row("Sort Google Fonts by", "Default order for the font catalog.",
        widgets::opts(&[("popular", "Popular"), ("trending", "Trending"), ("name", "Name"), ("newest", "Newest")]),
        &p.google_sort, |id| { prefs::update(|p| p.google_sort = id); events::emit(events::Change::Preferences); });
    g.add(&row);
    let (row, button) = widgets::button_row("Refresh the font lists", "", "Refresh now", |_| catalog::refresh());
    g.add(&row);
    let desc = row.first_child().and_then(|t| t.first_child()).and_then(|t| t.next_sibling()).and_downcast::<gtk::Label>();
    let label = widgets::label("", "settings-option-description");
    label.set_wrap(true);
    if let Some(text) = row.first_child().and_downcast::<gtk::Box>()
        && desc.is_none()
    {
        text.append(&label);
    }
    let refresh = {
        let (label, button) = (label.clone(), button.clone());
        move || {
            button.set_sensitive(!catalog::loading());
            let fetched = prefs::get().catalog_fetched;
            let when = if catalog::loading() {
                "Updating now…".to_string()
            } else if fetched == 0 {
                "Not fetched yet.".to_string()
            } else {
                format!("Last updated {}. They update once a day.", fmt::date(fetched))
            };
            label.set_text(&format!(
                "{} Google Fonts families and {} Nerd Fonts ({}). {when}",
                fmt::thousands(catalog::google().len()),
                catalog::nerd().len(),
                if catalog::nerd_release().is_empty() { "release unknown".into() } else { catalog::nerd_release() },
            ));
        }
    };
    refresh();
    events::subscribe(&label, move |c| {
        if c == events::Change::Catalog {
            refresh();
        }
    });
}
