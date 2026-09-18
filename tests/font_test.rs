use iced::advanced::text::{Paragraph as _, Text};
use iced::advanced::graphics::text::Paragraph;
use iced::{Font, Pixels, Size};

#[test]
fn test_font_cell_alignment() {
    let font_size = Pixels(14.0);
    let sample = "MMMMMMMMMM0123456789";
    let text = Text {
        content: sample,
        bounds: Size::INFINITY,
        size: font_size,
        line_height: iced::advanced::text::LineHeight::Relative(1.0),
        font: Font::MONOSPACE,
        horizontal_alignment: iced::alignment::Horizontal::Left,
        vertical_alignment: iced::alignment::Vertical::Top,
        shaping: iced::advanced::text::Shaping::Basic,
    };
    let p = Paragraph::with_text(text);
    let cell_w = p.min_width() / 20.0;

    let t_single = Text {
        content: "M",
        bounds: Size::INFINITY,
        size: font_size,
        line_height: iced::advanced::text::LineHeight::Relative(1.0),
        font: Font::MONOSPACE,
        horizontal_alignment: iced::alignment::Horizontal::Left,
        vertical_alignment: iced::alignment::Vertical::Top,
        shaping: iced::advanced::text::Shaping::Basic,
    };
    let p_single = Paragraph::with_text(t_single);
    let single_w = p_single.min_width();

    let diff = (single_w - cell_w).abs();
    assert!(diff < 0.001, "Discrepancy: single_w={}, cell_w={}", single_w, cell_w);
}
