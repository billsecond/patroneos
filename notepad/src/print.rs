//! Printing a tab's text in Notepad's font (at its own size, not the tab's
//! zoom), wrapped to the page, with the file's name at the top of each page
//! and the page's number at the bottom.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::pango;
use gtk::prelude::*;

#[derive(Default)]
struct Pages {
    layout: Option<pango::Layout>,
    /// Each line's baseline, top and bottom, in Pango units from the top
    /// of the text.
    lines: Vec<(i32, i32, i32)>,
    /// Each page's first line, and where the page starts in the text.
    pages: Vec<(usize, i32)>,
    /// The height kept at the top and bottom of each page for its name and
    /// number, in the print context's units.
    band: f64,
}

fn band_layout(context: &gtk::PrintContext, text: &str) -> pango::Layout {
    let layout = context.create_pango_layout();
    layout.set_font_description(Some(&pango::FontDescription::from_string("sans 9")));
    layout.set_width((context.width() * pango::SCALE as f64) as i32);
    layout.set_alignment(pango::Alignment::Center);
    layout.set_ellipsize(pango::EllipsizeMode::Middle);
    layout.set_text(text);
    layout
}

pub fn print(
    parent: &gtk::ApplicationWindow,
    title: &str,
    text: &str,
    font: &pango::FontDescription,
) {
    let operation = gtk::PrintOperation::new();
    operation.set_job_name(title);
    operation.set_embed_page_setup(true);
    let state = Rc::new(RefCell::new(Pages::default()));

    let pages = state.clone();
    let text = text.to_owned();
    let font = font.clone();
    let name = title.to_owned();
    operation.connect_begin_print(move |operation, context| {
        let layout = context.create_pango_layout();
        layout.set_font_description(Some(&font));
        layout.set_width((context.width() * pango::SCALE as f64) as i32);
        layout.set_wrap(pango::WrapMode::WordChar);
        layout.set_text(&text);
        let (_, band_height) = band_layout(context, &name).pixel_size();
        let band = band_height as f64 * 2.0;
        let page_height = ((context.height() - band * 2.0).max(1.0) * pango::SCALE as f64) as i32;

        let mut lines = Vec::new();
        let mut iter = layout.iter();
        loop {
            let (_, logical) = iter.line_extents();
            lines.push((iter.baseline(), logical.y(), logical.y() + logical.height()));
            if !iter.next_line() {
                break;
            }
        }
        let mut breaks = vec![(0, 0)];
        for (index, &(_, top, bottom)) in lines.iter().enumerate() {
            let &(first, page_top) = breaks.last().expect("there's always a first page");
            if bottom - page_top > page_height && index > first {
                breaks.push((index, top));
            }
        }
        operation.set_n_pages(breaks.len() as i32);
        let mut pages = pages.borrow_mut();
        pages.layout = Some(layout);
        pages.lines = lines;
        pages.pages = breaks;
        pages.band = band;
    });

    let pages = state.clone();
    let name = title.to_owned();
    operation.connect_draw_page(move |_, context, page| {
        let pages = pages.borrow();
        let (Some(layout), Some(&(first, page_top))) =
            (pages.layout.as_ref(), pages.pages.get(page as usize))
        else {
            return;
        };
        let last = pages
            .pages
            .get(page as usize + 1)
            .map(|&(next, _)| next)
            .unwrap_or(pages.lines.len());
        let cairo = context.cairo_context();
        cairo.set_source_rgb(0.0, 0.0, 0.0);

        let header = band_layout(context, &name);
        cairo.move_to(0.0, 0.0);
        pangocairo::functions::show_layout(&cairo, &header);
        let footer = band_layout(
            context,
            &format!("Page {} of {}", page + 1, pages.pages.len()),
        );
        let (_, footer_height) = footer.pixel_size();
        cairo.move_to(0.0, context.height() - footer_height as f64);
        pangocairo::functions::show_layout(&cairo, &footer);

        for index in first..last {
            let (Some(line), Some(&(baseline, _, _))) =
                (layout.line_readonly(index as i32), pages.lines.get(index))
            else {
                continue;
            };
            cairo.move_to(
                0.0,
                pages.band + (baseline - page_top) as f64 / pango::SCALE as f64,
            );
            pangocairo::functions::show_layout_line(&cairo, &line);
        }
    });

    if let Err(error) = operation.run(gtk::PrintOperationAction::PrintDialog, Some(parent)) {
        eprintln!("notepad: couldn't print: {error}");
    }
}
