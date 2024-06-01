use adw::prelude::*;
use relm4::factory::{DynamicIndex, FactoryComponent};
use relm4::FactorySender;

use std::fmt::Display;

use std::path::PathBuf;

#[derive(Debug, Clone)]

pub enum EntryType {
    Directory,
    File,
}

#[derive(Debug, Clone)]
pub struct PasswordStoreEntry {
    pub path: PathBuf,
    pub entry_type: EntryType,
}

#[derive(Debug, Clone)]
pub enum PasswordStoreEntryInput {
    Open,
}

#[derive(Debug, Clone)]
pub enum PasswordStoreEntryOutput {
    Open(PasswordStoreEntry),
}

impl From<PathBuf> for PasswordStoreEntry {
    fn from(path: PathBuf) -> Self {
        Self {
            entry_type: match path.is_dir() {
                true => EntryType::Directory,
                false => EntryType::File,
            },
            path,
        }
    }
}

impl From<PasswordStoreEntry> for PathBuf {
    fn from(entry: PasswordStoreEntry) -> Self {
        entry.path
    }
}

impl From<PasswordStoreEntry> for String {
    fn from(entry: PasswordStoreEntry) -> Self {
        entry.path.to_string_lossy().into()
    }
}

impl Display for PasswordStoreEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.to_string_lossy())
    }
}
#[relm4::factory(pub)]
impl FactoryComponent for PasswordStoreEntry {
    type Init = PathBuf;
    type Input = PasswordStoreEntryInput;
    type Output = PasswordStoreEntryOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        tracing::debug!("Initializing entry: {:?}", init);
        init.into()
    }

    view! {
        adw::ActionRow {
            set_title: &self.to_string(),
            set_activatable: true,
            set_selectable: false,
            connect_activated => PasswordStoreEntryInput::Open,
        }
    }

    #[tracing::instrument(skip_all)]
    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            PasswordStoreEntryInput::Open => {
                tracing::info!("Clicked entry, {:#?}", self);
                let open_output = PasswordStoreEntryOutput::Open(self.clone());
                sender.output(open_output).unwrap();
            }
        }
    }
}
