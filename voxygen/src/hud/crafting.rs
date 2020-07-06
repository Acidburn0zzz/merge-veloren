use super::{img_ids::Imgs, Show, TEXT_COLOR, UI_HIGHLIGHT_0, UI_MAIN};

use crate::{i18n::VoxygenLocalization, ui::fonts::ConrodVoxygenFonts};
use client::{self, Client};
use conrod_core::{
    color,
    widget::{self, Button, Image, Rectangle, Scrollbar, Text},
    widget_ids, Colorable, Labelable, Positionable, Sizeable, Widget, WidgetCommon,
};

widget_ids! {
    pub struct Ids {
        window,
        window_frame,
        close,
        icon,
        title_main,
        title_rec,
        align_rec,
        scrollbar_rec,
        title_ing,
        align_ing,
        scrollbar_ing,
        btn_craft,
    }
}

pub enum Event {
    Close,
}

#[derive(WidgetCommon)]
pub struct Crafting<'a> {
    show: &'a Show,
    client: &'a Client,
    imgs: &'a Imgs,
    fonts: &'a ConrodVoxygenFonts,
    localized_strings: &'a std::sync::Arc<VoxygenLocalization>,

    #[conrod(common_builder)]
    common: widget::CommonBuilder,
}

impl<'a> Crafting<'a> {
    pub fn new(
        show: &'a Show,
        client: &'a Client,
        imgs: &'a Imgs,
        fonts: &'a ConrodVoxygenFonts,
        localized_strings: &'a std::sync::Arc<VoxygenLocalization>,
    ) -> Self {
        Self {
            show,
            client,
            imgs,
            fonts,
            localized_strings,
            common: widget::CommonBuilder::default(),
        }
    }
}

impl<'a> Widget for Crafting<'a> {
    type Event = Vec<Event>;
    type State = Ids;
    type Style = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State { Ids::new(id_gen) }

    #[allow(clippy::unused_unit)] // TODO: Pending review in #587
    fn style(&self) -> Self::Style { () }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs {
            /* id, */ state: ids,
            ui,
            ..
        } = args;

        let mut events = Vec::new();

        Image::new(self.imgs.crafting_window)
            //.top_left_with_margins_on(ui.window, 200.0, 25.0)
            .middle_of(ui.window)
            .color(Some(UI_MAIN))
            .w_h(422.0, 460.0)
            .set(ids.window, ui);
        Image::new(self.imgs.crafting_frame)
            .middle_of(ids.window)
            .color(Some(UI_HIGHLIGHT_0))
            .w_h(422.0, 460.0)
            .set(ids.window_frame, ui);

        //  Close Button
        if Button::image(self.imgs.close_button)
            .w_h(24.0, 25.0)
            .hover_image(self.imgs.close_button_hover)
            .press_image(self.imgs.close_button_press)
            .top_right_with_margins_on(ids.window, 0.0, 0.0)
            .set(ids.close, ui)
            .was_clicked()
        {
            events.push(Event::Close);
        }

        // Title
        Text::new(&self.localized_strings.get("hud.crafting"))
            .mid_top_with_margin_on(ids.window_frame, 9.0)
            .font_id(self.fonts.cyri.conrod_id)
            .font_size(self.fonts.cyri.scale(22))
            .color(TEXT_COLOR)
            .set(ids.title_main, ui);

        // Alignment
        Rectangle::fill_with([136.0, 378.0], color::TRANSPARENT)
            .top_left_with_margins_on(ids.window_frame, 74.0, 5.0)
            .set(ids.align_rec, ui);
        Rectangle::fill_with([274.0, 340.0], color::TRANSPARENT)
            .top_right_with_margins_on(ids.window, 74.0, 5.0)
            .set(ids.align_ing, ui);

        // Add recipes as array of Button::image(nothing) with a widget_id generator
        // here and align to align_rec

        // Add ingredients here and align to align_ing

        // Scrollbars
        Scrollbar::y_axis(ids.align_rec)
            .thickness(5.0)
            .rgba(0.33, 0.33, 0.33, 1.0)
            .set(ids.scrollbar_rec, ui);
        Scrollbar::y_axis(ids.align_ing)
            .thickness(5.0)
            .rgba(0.33, 0.33, 0.33, 1.0)
            .set(ids.scrollbar_ing, ui);

        // Title Recipes and Ingredients
        Text::new(&self.localized_strings.get("hud.crafting.recipes"))
            .mid_top_with_margin_on(ids.align_rec, -22.0)
            .font_id(self.fonts.cyri.conrod_id)
            .font_size(self.fonts.cyri.scale(14))
            .color(TEXT_COLOR)
            .parent(ids.window)
            .set(ids.title_rec, ui);
        Text::new(&self.localized_strings.get("hud.crafting.ingredients"))
            .mid_top_with_margin_on(ids.align_ing, -22.0)
            .font_id(self.fonts.cyri.conrod_id)
            .font_size(self.fonts.cyri.scale(14))
            .color(TEXT_COLOR)
            .parent(ids.window)
            .set(ids.title_ing, ui);

        // Craft button
        if Button::image(self.imgs.button)
            .w_h(105.0, 25.0)
            .hover_image(self.imgs.button_hover)
            .press_image(self.imgs.button_press)
            .label(&self.localized_strings.get("hud.crafting.craft"))
            .label_y(conrod_core::position::Relative::Scalar(1.0))
            .label_color(TEXT_COLOR)
            .label_font_size(self.fonts.cyri.scale(12))
            .label_font_id(self.fonts.cyri.conrod_id)
            .mid_bottom_with_margin_on(ids.align_ing, -31.0)
            .set(ids.btn_craft, ui)
            .was_clicked()
        {
            // CRAFT!
        }

        events
    }
}
