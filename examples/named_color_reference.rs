//! A simple example presenting all of the named colors in alphabetical order.
//!
//! This is also used as a test for nannou developers to test that colors specified via the `Ui`
//! and `Draw` API behave as expected, and easily compare them to the online css reference:
//! https://www.w3schools.com/cssref/css_colors.asp

use nannou::prelude::*;
use nannou::ui::position::{Place, Relative};
use nannou::ui::prelude::*;

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

struct Model {
    ui: Ui,
    color_list: widget::Id,
    selected_color_index: usize,
}

fn model(app: &App) -> Model {
    check_color_list_lengths();
    app.set_loop_mode(LoopMode::Wait);
    let mut ui = app.new_ui().build().unwrap();
    let color_list = ui.generate_widget_id();
    let selected_color_index = 0;
    Model {
        ui,
        color_list,
        selected_color_index,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let Model {
        ref mut ui,
        ref mut selected_color_index,
        color_list,
    } = *model;

    // Calling `set_widgets` allows us to instantiate some widgets.
    let ui = &mut ui.set_widgets();

    let win_rect = app.main_window().rect();

    // The list of colours.
    let (mut events, scrollbar) = widget::ListSelect::single(ALL_NAMED_COLORS.len())
        .flow_down()
        .item_size(30.0)
        .scrollbar_on_top()
        .w_h(200.0, win_rect.h() as _)
        .top_left()
        .set(color_list, ui);

    while let Some(event) = events.next(ui, |i| i == *selected_color_index) {
        use nannou::ui::widget::list_select::Event;
        match event {
            Event::Item(item) => {
                let label = &ALL_NAMED_COLOR_NAMES[item.i];
                let (r, g, b) = ALL_NAMED_COLORS[item.i].into();
                let color = nannou::ui::color::rgb_bytes(r, g, b);
                let button = widget::Button::new()
                    .border(0.0)
                    .color(color)
                    .label(label)
                    .label_font_size(10)
                    .label_x(Relative::Place(Place::Start(Some(20.0))))
                    .label_color(color.plain_contrast());
                item.set(button, ui);
            }

            Event::Selection(new_index) => *selected_color_index = new_index,

            _ => (),
        }
    }

    if let Some(scrollbar) = scrollbar {
        scrollbar.set(ui);
    }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model, frame: &Frame) {
    let draw = app.draw();

    // Draw the background with the color.
    draw.background()
        .color(ALL_NAMED_COLORS[model.selected_color_index]);

    // Also draw a rectangle with the same color to ensure our vertex data is accurate too!
    // If we can see this rectangle on the bottom half of the window, something has gone wrong.
    let win = app.main_window().rect();
    draw.rect()
        .w_h(win.w(), win.h() * 0.5)
        .x_y(0.0, -win.h() * 0.25)
        .color(ALL_NAMED_COLORS[model.selected_color_index]);

    // Clear the background and draw the rect.
    draw.to_frame(app, &frame).unwrap();

    // Draw the color list to the frame.
    model.ui.draw_to_frame(app, &frame).unwrap();
}

fn check_color_list_lengths() {
    assert_eq!(
        ALL_NAMED_COLORS.len(),
        ALL_NAMED_COLOR_NAMES.len(),
        "Woops! It looks like either `ALL_NAMED_COLORS` or `ALL_NAMED_COLOR_NAMES` was updated \
         without updating the other!",
    )
}

pub const ALL_NAMED_COLORS: &[nannou::color::Srgb<u8>] = &[
    ALICEBLUE,
    ANTIQUEWHITE,
    AQUA,
    AQUAMARINE,
    AZURE,
    BEIGE,
    BISQUE,
    BLACK,
    BLANCHEDALMOND,
    BLUE,
    BLUEVIOLET,
    BROWN,
    BURLYWOOD,
    CADETBLUE,
    CHARTREUSE,
    CHOCOLATE,
    CORAL,
    CORNFLOWERBLUE,
    CORNSILK,
    CRIMSON,
    CYAN,
    DARKBLUE,
    DARKCYAN,
    DARKGOLDENROD,
    DARKGRAY,
    DARKGREEN,
    DARKGREY,
    DARKKHAKI,
    DARKMAGENTA,
    DARKOLIVEGREEN,
    DARKORANGE,
    DARKORCHID,
    DARKRED,
    DARKSALMON,
    DARKSEAGREEN,
    DARKSLATEBLUE,
    DARKSLATEGRAY,
    DARKSLATEGREY,
    DARKTURQUOISE,
    DARKVIOLET,
    DEEPPINK,
    DEEPSKYBLUE,
    DIMGRAY,
    DIMGREY,
    DODGERBLUE,
    FIREBRICK,
    FLORALWHITE,
    FORESTGREEN,
    FUCHSIA,
    GAINSBORO,
    GHOSTWHITE,
    GOLD,
    GOLDENROD,
    GRAY,
    GREEN,
    GREENYELLOW,
    GREY,
    HONEYDEW,
    HOTPINK,
    INDIANRED,
    INDIGO,
    IVORY,
    KHAKI,
    LAVENDER,
    LAVENDERBLUSH,
    LAWNGREEN,
    LEMONCHIFFON,
    LIGHTBLUE,
    LIGHTCORAL,
    LIGHTCYAN,
    LIGHTGOLDENRODYELLOW,
    LIGHTGRAY,
    LIGHTGREEN,
    LIGHTGREY,
    LIGHTPINK,
    LIGHTSALMON,
    LIGHTSEAGREEN,
    LIGHTSKYBLUE,
    LIGHTSLATEGRAY,
    LIGHTSLATEGREY,
    LIGHTSTEELBLUE,
    LIGHTYELLOW,
    LIME,
    LIMEGREEN,
    LINEN,
    MAGENTA,
    MAROON,
    MEDIUMAQUAMARINE,
    MEDIUMBLUE,
    MEDIUMORCHID,
    MEDIUMPURPLE,
    MEDIUMSEAGREEN,
    MEDIUMSLATEBLUE,
    MEDIUMSPRINGGREEN,
    MEDIUMTURQUOISE,
    MEDIUMVIOLETRED,
    MIDNIGHTBLUE,
    MINTCREAM,
    MISTYROSE,
    MOCCASIN,
    NAVAJOWHITE,
    NAVY,
    OLDLACE,
    OLIVE,
    OLIVEDRAB,
    ORANGE,
    ORANGERED,
    ORCHID,
    PALEGOLDENROD,
    PALEGREEN,
    PALETURQUOISE,
    PALEVIOLETRED,
    PAPAYAWHIP,
    PEACHPUFF,
    PERU,
    PINK,
    PLUM,
    POWDERBLUE,
    PURPLE,
    REBECCAPURPLE,
    RED,
    ROSYBROWN,
    ROYALBLUE,
    SADDLEBROWN,
    SALMON,
    SANDYBROWN,
    SEAGREEN,
    SEASHELL,
    SIENNA,
    SILVER,
    SKYBLUE,
    SLATEBLUE,
    SLATEGRAY,
    SLATEGREY,
    SNOW,
    SPRINGGREEN,
    STEELBLUE,
    TAN,
    TEAL,
    THISTLE,
    TOMATO,
    TURQUOISE,
    VIOLET,
    WHEAT,
    WHITE,
    WHITESMOKE,
    YELLOW,
    YELLOWGREEN,
];

pub const ALL_NAMED_COLOR_NAMES: &[&str] = &[
    "ALICEBLUE",
    "ANTIQUEWHITE",
    "AQUA",
    "AQUAMARINE",
    "AZURE",
    "BEIGE",
    "BISQUE",
    "BLACK",
    "BLANCHEDALMOND",
    "BLUE",
    "BLUEVIOLET",
    "BROWN",
    "BURLYWOOD",
    "CADETBLUE",
    "CHARTREUSE",
    "CHOCOLATE",
    "CORAL",
    "CORNFLOWERBLUE",
    "CORNSILK",
    "CRIMSON",
    "CYAN",
    "DARKBLUE",
    "DARKCYAN",
    "DARKGOLDENROD",
    "DARKGRAY",
    "DARKGREEN",
    "DARKGREY",
    "DARKKHAKI",
    "DARKMAGENTA",
    "DARKOLIVEGREEN",
    "DARKORANGE",
    "DARKORCHID",
    "DARKRED",
    "DARKSALMON",
    "DARKSEAGREEN",
    "DARKSLATEBLUE",
    "DARKSLATEGRAY",
    "DARKSLATEGREY",
    "DARKTURQUOISE",
    "DARKVIOLET",
    "DEEPPINK",
    "DEEPSKYBLUE",
    "DIMGRAY",
    "DIMGREY",
    "DODGERBLUE",
    "FIREBRICK",
    "FLORALWHITE",
    "FORESTGREEN",
    "FUCHSIA",
    "GAINSBORO",
    "GHOSTWHITE",
    "GOLD",
    "GOLDENROD",
    "GRAY",
    "GREEN",
    "GREENYELLOW",
    "GREY",
    "HONEYDEW",
    "HOTPINK",
    "INDIANRED",
    "INDIGO",
    "IVORY",
    "KHAKI",
    "LAVENDER",
    "LAVENDERBLUSH",
    "LAWNGREEN",
    "LEMONCHIFFON",
    "LIGHTBLUE",
    "LIGHTCORAL",
    "LIGHTCYAN",
    "LIGHTGOLDENRODYELLOW",
    "LIGHTGRAY",
    "LIGHTGREEN",
    "LIGHTGREY",
    "LIGHTPINK",
    "LIGHTSALMON",
    "LIGHTSEAGREEN",
    "LIGHTSKYBLUE",
    "LIGHTSLATEGRAY",
    "LIGHTSLATEGREY",
    "LIGHTSTEELBLUE",
    "LIGHTYELLOW",
    "LIME",
    "LIMEGREEN",
    "LINEN",
    "MAGENTA",
    "MAROON",
    "MEDIUMAQUAMARINE",
    "MEDIUMBLUE",
    "MEDIUMORCHID",
    "MEDIUMPURPLE",
    "MEDIUMSEAGREEN",
    "MEDIUMSLATEBLUE",
    "MEDIUMSPRINGGREEN",
    "MEDIUMTURQUOISE",
    "MEDIUMVIOLETRED",
    "MIDNIGHTBLUE",
    "MINTCREAM",
    "MISTYROSE",
    "MOCCASIN",
    "NAVAJOWHITE",
    "NAVY",
    "OLDLACE",
    "OLIVE",
    "OLIVEDRAB",
    "ORANGE",
    "ORANGERED",
    "ORCHID",
    "PALEGOLDENROD",
    "PALEGREEN",
    "PALETURQUOISE",
    "PALEVIOLETRED",
    "PAPAYAWHIP",
    "PEACHPUFF",
    "PERU",
    "PINK",
    "PLUM",
    "POWDERBLUE",
    "PURPLE",
    "REBECCAPURPLE",
    "RED",
    "ROSYBROWN",
    "ROYALBLUE",
    "SADDLEBROWN",
    "SALMON",
    "SANDYBROWN",
    "SEAGREEN",
    "SEASHELL",
    "SIENNA",
    "SILVER",
    "SKYBLUE",
    "SLATEBLUE",
    "SLATEGRAY",
    "SLATEGREY",
    "SNOW",
    "SPRINGGREEN",
    "STEELBLUE",
    "TAN",
    "TEAL",
    "THISTLE",
    "TOMATO",
    "TURQUOISE",
    "VIOLET",
    "WHEAT",
    "WHITE",
    "WHITESMOKE",
    "YELLOW",
    "YELLOWGREEN",
];
