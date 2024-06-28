use std::fs::DirEntry;
use std::path::PathBuf;

use adw::prelude::*;

use adw::{ApplicationWindow, HeaderBar};
use gtk::{Box, Orientation, SelectionMode};

use relm4::prelude::*;

use relm4::factory::FactoryVecDeque;

use crate::ui::list_entry;
use list_entry::PasswordStoreEntry;

use super::list_entry::EntryType;

pub struct App {
    pub counter: u8,
    pub store_root: PathBuf,
    pub current_dir: PathBuf,
    pub visible_files: FactoryVecDeque<PasswordStoreEntry>,
}

#[derive(Debug)]
pub enum Msg {
    Increment,
    Decrement,
    TravelUp,
    OpenPath(PasswordStoreEntry),
    UpdateEntries,
}

#[relm4::component(pub)]
impl SimpleComponent for App {
    type Init = u8;
    type Input = Msg;
    type Output = ();

    view! {
        ApplicationWindow {
            set_title: Some("Simple app"),
            set_default_size: (300, 100),

            Box {
                set_orientation: Orientation::Vertical,

                HeaderBar,

                gtk::Box {
                    set_orientation: Orientation::Vertical,
                    set_spacing: 5,
                    set_margin_all: 5,

                    gtk::ScrolledWindow {
                        set_hscrollbar_policy: gtk::PolicyType::Never,
                        set_min_content_height: 360,
                        set_vexpand: true,

                        #[local_ref]
                        task_list_box -> gtk::ListBox {
                            set_selection_mode: SelectionMode::None,
                            set_css_classes: &[ "content" ],
                        }
                    },

                    gtk::Button {
                        set_label: "Increment",
                        connect_clicked => Msg::Increment,
                    },

                    gtk::Button {
                        set_label: "Decrement",
                        connect_clicked => Msg::Decrement,
                    },

                    gtk::Label {
                        #[watch]
                        set_label: &format!("Counter: {}", model.counter),
                        set_margin_all: 5,
                    }
                }
            }

        }
    }

    // Initialize the component.
    fn init(
        counter: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let password_store_dir = std::env::var("PASSWORD_STORE_DIR").unwrap_or(
            // $HOME/.password-store
            format!(
                "{}/.password-store",
                dirs::home_dir().unwrap().to_str().unwrap()
            ),
        );

        tracing::info!("password store directory: {}", password_store_dir,);

        let store_root = PathBuf::from(password_store_dir).clone();

        let visible_files =
            FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |output| match output {
                    list_entry::PasswordStoreEntryOutput::Open(entry) => Msg::OpenPath(entry),
                });

        tracing::debug!("Constructing main window model");
        let model = App {
            counter,
            current_dir: store_root.clone(),
            store_root,
            visible_files,
        };

        sender.input(Msg::UpdateEntries);

        let task_list_box = model.visible_files.widget();
        // Insert the code generation of the view! macro here
        let widgets = view_output!();

        tracing::debug!("Finished init");
        ComponentParts { model, widgets }
    }

    // #[tracing::instrument(level = "debug", skip_all, fields(self))]
    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            Msg::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
            Msg::Decrement => {
                self.counter = self.counter.wrapping_sub(1);
            }
            Msg::TravelUp => {
                tracing::info_span!("Traveling up").in_scope(|| {
                    if self.current_dir == self.store_root {
                        tracing::warn!("Attempted to travel up from root of store",);
                        return;
                    }
                    tracing::debug!("Current dir: {}", self.current_dir.display());
                    match self.current_dir.parent() {
                        Some(v) => {
                            self.current_dir = v.into();
                            tracing::debug!("New current dir: {}", self.current_dir.display());
                        }
                        None => {
                            tracing::error!("Can't travel up, already at root????",)
                        }
                    }
                });
            }
            Msg::OpenPath(entry) => {
                tracing::info!("Opening path: {}", entry.path.display());
                match entry.entry_type {
                    EntryType::Directory => {
                        tracing::info!("Opening directory: {}", entry.path.display());
                        self.current_dir = entry.path;
                        sender.input(Msg::UpdateEntries);
                    }
                    _ => {
                        tracing::error!("Openning non-directory entry is not supported",)
                    }
                };
            }
            Msg::UpdateEntries => {
                tracing::debug!("Filling entries");

                let visible_files_entries = std::fs::read_dir(&self.current_dir)
                    .unwrap()
                    .map(Result::unwrap)
                    .map(|arg0: DirEntry| DirEntry::path(&arg0))
                    .filter(|path| {
                        let basename = path.file_name().unwrap_or_default().to_string_lossy();
                        let hidden = basename.starts_with('.');

                        !hidden && (path.is_dir() || basename.ends_with(".gpg"))
                    })
                    .map(PasswordStoreEntry::from)
                    .collect::<Vec<PasswordStoreEntry>>();

                tracing::info!("{} visible files", visible_files_entries.len());

                let mut guard = self.visible_files.guard();
                guard.clear();
                for entry in visible_files_entries {
                    guard.push_back(entry.path);
                }
            }
        }
    }
}
