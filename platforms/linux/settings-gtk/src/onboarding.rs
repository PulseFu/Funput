//! First-run onboarding: a 4-step `AdwCarousel` wizard. Ports the steps/copy of the
//! retired Svelte onboarding (`platforms/ui/src/lib/onboarding/Onboarding.svelte`),
//! Linux variant. On finish it marks `has_completed_onboarding` and closes; the next
//! launch opens Settings directly.

use std::rc::Rc;

use adw::prelude::*;
use adw::Application;
use gtk::{Align, Justification, Orientation};

use crate::settings::{Method, Settings};

const STEPS: u32 = 4;

pub fn build(app: &Application) -> adw::Window {
    let window = adw::Window::builder()
        .title("Chào mừng đến Funput")
        .default_width(460)
        .default_height(560)
        .build();
    window.set_application(Some(app));

    let carousel = adw::Carousel::new();
    carousel.set_vexpand(true);
    carousel.append(&welcome_step());
    carousel.append(&method_step());
    carousel.append(&how_step());
    carousel.append(&ready_step());

    let dots = adw::CarouselIndicatorDots::new();
    dots.set_carousel(Some(&carousel));

    // Navigation buttons.
    let back = gtk::Button::with_label("Quay lại");
    let next = gtk::Button::with_label("Tiếp tục");
    next.add_css_class("suggested-action");

    let spacer = gtk::Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    let nav = gtk::Box::new(Orientation::Horizontal, 8);
    nav.append(&back);
    nav.append(&spacer);
    nav.append(&next);

    // Keep the buttons in sync with the visible page.
    let update: Rc<dyn Fn(u32)> = {
        let back = back.clone();
        let next = next.clone();
        Rc::new(move |idx: u32| {
            back.set_visible(idx > 0);
            next.set_label(if idx + 1 < STEPS { "Tiếp tục" } else { "Bắt đầu" });
        })
    };
    update(0);
    let update_sig = update.clone();
    carousel.connect_page_changed(move |_, idx| update_sig(idx));

    let carousel_next = carousel.clone();
    let window_next = window.clone();
    next.connect_clicked(move |_| {
        let pos = carousel_next.position().round() as u32;
        if pos + 1 < STEPS {
            let page = carousel_next.nth_page(pos + 1);
            carousel_next.scroll_to(&page, true);
        } else {
            Settings::update(|s| s.has_completed_onboarding = true);
            window_next.close();
        }
    });
    let carousel_back = carousel.clone();
    back.connect_clicked(move |_| {
        let pos = carousel_back.position().round() as u32;
        if pos > 0 {
            let page = carousel_back.nth_page(pos - 1);
            carousel_back.scroll_to(&page, true);
        }
    });

    let content = gtk::Box::new(Orientation::Vertical, 16);
    content.set_margin_top(16);
    content.set_margin_bottom(16);
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.append(&carousel);
    content.append(&dots);
    content.append(&nav);

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&adw::HeaderBar::new());
    toolbar.set_content(Some(&content));
    window.set_content(Some(&toolbar));

    window
}

/// A centered title/body step. `emoji` is rendered large via Pango markup.
fn step(emoji: &str, title: &str, body: &str) -> gtk::Box {
    let b = gtk::Box::new(Orientation::Vertical, 12);
    b.set_valign(Align::Center);
    b.set_halign(Align::Center);
    b.set_margin_start(24);
    b.set_margin_end(24);

    let hero = gtk::Label::new(None);
    hero.set_markup(&format!("<span size=\"xx-large\">{emoji}</span>"));

    let title_label = gtk::Label::new(Some(title));
    title_label.add_css_class("title-2");

    let body_label = gtk::Label::new(Some(body));
    body_label.add_css_class("dim-label");
    body_label.set_wrap(true);
    body_label.set_justify(Justification::Center);

    b.append(&hero);
    b.append(&title_label);
    b.append(&body_label);
    b
}

fn welcome_step() -> gtk::Box {
    step(
        "👋",
        "Chào mừng đến Funput",
        "Gõ tiếng Việt ở mọi nơi trên Linux — miễn phí, mã nguồn mở.",
    )
}

fn method_step() -> gtk::Box {
    let b = step("⌨️", "Chọn kiểu gõ", "Có thể đổi bất cứ lúc nào trong Cài đặt.");

    let s = Settings::load();
    let telex = gtk::ToggleButton::with_label("Telex");
    let vni = gtk::ToggleButton::with_label("VNI");
    vni.set_group(Some(&telex));
    match s.method {
        Method::Telex => telex.set_active(true),
        Method::Vni => vni.set_active(true),
    }
    telex.connect_toggled(|btn| {
        if btn.is_active() {
            Settings::update(|s| s.method = Method::Telex);
        }
    });
    vni.connect_toggled(|btn| {
        if btn.is_active() {
            Settings::update(|s| s.method = Method::Vni);
        }
    });

    let picker = gtk::Box::new(Orientation::Horizontal, 0);
    picker.add_css_class("linked");
    picker.set_halign(Align::Center);
    picker.append(&telex);
    picker.append(&vni);
    b.append(&picker);
    b
}

fn how_step() -> gtk::Box {
    step(
        "🔔",
        "Cách hoạt động",
        "Funput chạy trong bộ gõ của hệ thống. Nhấn Ctrl + ` để bật/tắt nhanh tiếng Việt.",
    )
}

fn ready_step() -> gtk::Box {
    step(
        "✅",
        "Sẵn sàng!",
        "Bật Funput trong fcitx5-configtool (Fcitx5) hoặc Cài đặt → Input Sources (IBus), rồi gõ thử ngay.",
    )
}
