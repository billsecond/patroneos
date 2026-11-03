//! Line numbers, in a narrow column at the text's left (View › Line
//! numbers): each line's number beside its first row, in the text's own
//! font and size, faint. A wrapped line has one number.

use gtk::pango;
use gtk::prelude::*;

/// The column, for `view`. It draws itself as the text scrolls or changes.
pub fn new(view: &gtk::TextView, scroller: &gtk::ScrolledWindow) -> gtk::DrawingArea {
    let area = gtk::DrawingArea::builder()
        .css_classes(["line-numbers"])
        .build();
    let text = view.clone();
    area.set_draw_func(move |area, cairo, width, _| {
        let buffer = text.buffer();
        let layout = area.create_pango_layout(None);
        let mut color = area.color();
        color.set_alpha(color.alpha() * 0.45);
        cairo.set_source_rgba(
            color.red() as f64,
            color.green() as f64,
            color.blue() as f64,
            color.alpha() as f64,
        );
        let visible = text.visible_rect();
        let (mut line, _) = text.line_at_y(visible.y());
        let current = buffer.iter_at_mark(&buffer.get_insert()).line();
        loop {
            let (y, _) = text.line_yrange(&line);
            if y > visible.y() + visible.height() {
                break;
            }
            let (_, top) = text.buffer_to_window_coords(gtk::TextWindowType::Left, 0, y);
            let number = line.line() + 1;
            layout.set_text(&number.to_string());
            let mut font = pango::FontDescription::new();
            font.set_weight(if line.line() == current {
                pango::Weight::Bold
            } else {
                pango::Weight::Normal
            });
            layout.set_font_description(Some(&font));
            // The line's first row: where its first character's drawn
            let first = text.iter_location(&line);
            let (_, row_top) =
                text.buffer_to_window_coords(gtk::TextWindowType::Left, 0, first.y());
            let (text_width, text_height) = layout.pixel_size();
            let row_height = first.height().max(text_height);
            cairo.move_to(
                (width - text_width - 10) as f64,
                (row_top.max(top) + (row_height - text_height) / 2) as f64,
            );
            pangocairo::functions::show_layout(cairo, &layout);
            if !line.forward_line() {
                break;
            }
        }
    });
    let redraw = {
        let area = area.clone();
        let text = view.clone();
        move || {
            // Room for the biggest number, and a little either side
            let digits = text.buffer().line_count().max(1).to_string().len().max(2);
            let layout = area.create_pango_layout(Some(&"0".repeat(digits)));
            let width = layout.pixel_size().0 + 20;
            if area.content_width() != width {
                area.set_content_width(width);
            }
            area.queue_draw();
        }
    };
    let on_scroll = redraw.clone();
    scroller
        .vadjustment()
        .connect_value_changed(move |_| on_scroll());
    let on_change = redraw.clone();
    view.buffer().connect_changed(move |_| on_change());
    let on_move = redraw.clone();
    view.buffer()
        .connect_cursor_position_notify(move |_| on_move());
    let on_resize = redraw.clone();
    scroller
        .hadjustment()
        .connect_page_size_notify(move |_| on_resize());
    // Its width from the start, not only once the text changes
    redraw();
    area
}
