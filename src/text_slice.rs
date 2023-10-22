
use gtk::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use gtk::glib::subclass::Signal;

#[repr(i32)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "TextStyle")]
pub enum TextStyle {
    HeadingOne,
    HeadingTwo,
    HeadingThree,
    HeadingFour,
    #[default]
    Normal,
}

mod imp {
    use super::*;

    use std::cell::Cell;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default)]
    pub struct TextSlice {
        pub style: Cell<TextStyle>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TextSlice {
        const NAME: &'static str = "TextSlice";
        type Type = super::TextSlice;
        type ParentType = gtk::TextView;
    }

    impl ObjectImpl for TextSlice {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            obj.set_wrap_mode(gtk::WrapMode::WordChar);
            obj.set_hexpand(true);

            obj.set_top_margin(9);
            obj.set_bottom_margin(9);
            obj.set_left_margin(9);
            obj.set_right_margin(9);

            obj.buffer().create_tag(Some(&"heading-one"), &[("size-points", &20.0), ("weight", &800)]);
            obj.buffer().create_tag(Some(&"heading-two"), &[("size-points", &15.0), ("weight", &800)]);
            obj.buffer().create_tag(Some(&"heading-three"), &[("size-points", &15.0), ("weight", &700)]);
            obj.buffer().create_tag(Some(&"heading-four"), &[("size-points", &13.0), ("weight", &700)]);
            obj.buffer().create_tag(Some(&"normal"), &[("size-points", &11.0), ("weight", &400)]);

            obj.buffer().connect_changed(
                glib::clone!(@weak self as this => move |buffer| {
                    let tag = match this.style.clone().get() {
                        TextStyle::HeadingOne => "heading-one",
                        TextStyle::HeadingTwo => "heading-two",
                        TextStyle::HeadingThree => "heading-three",
                        TextStyle::HeadingFour => "heading-four",
                        TextStyle::Normal => "normal",
                    };

                    buffer.apply_tag_by_name(tag, &buffer.start_iter(), &buffer.end_iter());
                })
            );
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("should-delete").build()]);
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for TextSlice {}
    impl TextViewImpl for TextSlice {
        fn backspace(&self) {
            self.parent_backspace();

            let obj = self.obj();

            let text = obj.buffer().text(&obj.buffer().start_iter(), &obj.buffer().end_iter(), false);

            if text.is_empty() {
                obj.emit_by_name::<()>("should-delete", &[]);
            }
        }
    }
}

glib::wrapper! {
    pub struct TextSlice(ObjectSubclass<imp::TextSlice>)
        @extends gtk::Widget, gtk::TextView,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl TextSlice {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn new_from_style(style: TextStyle) -> Self {
        let obj: TextSlice = glib::Object::new();
        obj.set_style(style);

        obj
    }

    pub fn set_style(&self, style: TextStyle) {
        let imp = self.imp();
        imp.style.replace(style);
    }
}
