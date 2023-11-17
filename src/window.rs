/* window.rs
 *
 * Copyright 2023 Felipe Kinoshita
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use gtk::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use gettextrs::gettext;

use crate::text_slice::{TextSlice, TextStyle};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/felipekinoshita/Sentence/window.ui")]
    pub struct SentenceWindow {
        // Template widgets
        #[template_child]
        pub navigation_view: TemplateChild<adw::NavigationView>,
        #[template_child]
        pub editor_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub editor_container: TemplateChild<gtk::Box>,
        #[template_child]
        pub editor_add_button: TemplateChild<gtk::MenuButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SentenceWindow {
        const NAME: &'static str = "SentenceWindow";
        type Type = super::SentenceWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SentenceWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            let imp = obj.imp();

            imp.editor_button.connect_clicked(
                glib::clone!(@weak self as this => move |_| {
                    this.navigation_view.push_by_tag("editor");
                })
            );

            obj.slices_from_markdown("# Lorem Ipsum\n\nThis is a paragraph\n\n## Subtitle\n\nasdjkalsdjkalsdklasd\n");

            // self.add_text_slice_with_placeholder(TextStyle::HeadingOne, &gettext("Title"));
        }
    }

    #[gtk::template_callbacks]
    impl SentenceWindow {
        fn add_text_slice(&self, style: TextStyle) {
            self.obj().add_text_slice_with_placeholder(style, "");
            self.obj().imp().editor_add_button.popdown();
        }

        #[template_callback]
        fn on_heading_one_clicked(&self, _button: gtk::Button) {
            self.add_text_slice(TextStyle::HeadingOne);
        }

        #[template_callback]
        fn on_heading_two_clicked(&self, _button: gtk::Button) {
            self.add_text_slice(TextStyle::HeadingTwo);
        }

        #[template_callback]
        fn on_heading_three_clicked(&self, _button: gtk::Button) {
            self.add_text_slice(TextStyle::HeadingThree);
        }

        #[template_callback]
        fn on_heading_four_clicked(&self, _button: gtk::Button) {
            self.add_text_slice(TextStyle::HeadingFour);
        }

        #[template_callback]
        fn on_normal_text_clicked(&self, _button: gtk::Button) {
            self.add_text_slice(TextStyle::Normal);
        }

        #[template_callback]
        fn on_generate_markdown_clicked(&self) {
            self.obj().generate_markdown();
        }
    }

    impl WidgetImpl for SentenceWindow {}
    impl WindowImpl for SentenceWindow {}
    impl ApplicationWindowImpl for SentenceWindow {}
    impl AdwApplicationWindowImpl for SentenceWindow {}
}

glib::wrapper! {
    pub struct SentenceWindow(ObjectSubclass<imp::SentenceWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,        @implements gio::ActionGroup, gio::ActionMap;
}

impl SentenceWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    fn add_text_slice_with_placeholder(&self, style: TextStyle, placeholder: &str) {
        let imp = self.imp();

        let id = imp.editor_container.observe_children().n_items() as usize;

        let thing = TextSlice::new_from_style(style);
        thing.buffer().set_text(placeholder);

        imp.editor_container.append(&thing);
        thing.grab_focus();

        thing.connect_local(
            "should-delete",
            false,
            glib::clone!(@weak self as this => @default-return None, move |_| {
                this.imp().editor_container
                    .observe_children()
                    .into_iter()
                    .for_each(|child| {
                        if let Ok(child) = child {
                            this.imp().editor_container.remove(child.downcast_ref::<gtk::Widget>().unwrap());
                        }
                    });

                None
            }),
        );
    }

    fn slices_from_markdown(&self, markdown: &str) {
        let imp = self.imp();

        imp.editor_container
            .observe_children()
            .into_iter()
            .for_each(|child| {
                if let Ok(child) = child {
                    imp.editor_container.remove(child.downcast_ref::<gtk::Widget>().unwrap());
                }
            });

        let ast = markdown::to_mdast(markdown, &markdown::ParseOptions::default()).expect("error from markdown ast");
        for child in ast.children().unwrap() {
            let node = child.children().unwrap().first().unwrap().clone();
            let mut node_text = "";

            match node {
                markdown::mdast::Node::Text(ref node) => {
                    node_text = &node.value;
                },
                _ => {},
            }

            match child {
                markdown::mdast::Node::Heading(node) => {
                    let style = match node.depth {
                        1 => TextStyle::HeadingOne,
                        2 => TextStyle::HeadingTwo,
                        3 => TextStyle::HeadingThree,
                        4 => TextStyle::HeadingFour,
                        _ => TextStyle::Normal,
                    };

                    self.add_text_slice_with_placeholder(style, node_text);
                }
                markdown::mdast::Node::Paragraph(_node) => {
                    self.add_text_slice_with_placeholder(TextStyle::Normal, node_text);
                }
                _ => {},
            }
        }
    }

    fn generate_markdown(&self) {
        let mut output = String::new();

        let imp = self.imp();

        imp.editor_container
            .observe_children()
            .into_iter()
            .for_each(|child| {
                if let Ok(c) = child {
                    let c = c.downcast_ref::<TextSlice>().unwrap();
                    let b = c.buffer();
                    let buffer = b.downcast_ref::<gtk::TextBuffer>().unwrap();
                    let text = buffer.text(&mut buffer.start_iter(), &mut buffer.end_iter(), false);

                    match c.style() {
                        TextStyle::HeadingOne => output.push_str(&format!("# {}\n\n", text)),
                        TextStyle::HeadingTwo => output.push_str(&format!("## {}\n\n", text)),
                        TextStyle::HeadingThree => output.push_str(&format!("### {}\n\n", text)),
                        TextStyle::HeadingFour => output.push_str(&format!("#### {}\n\n", text)),
                        TextStyle::Normal => output.push_str(&format!("{}\n\n", text)),
                    }
                }
            });

        println!("{output}");
    }
}
