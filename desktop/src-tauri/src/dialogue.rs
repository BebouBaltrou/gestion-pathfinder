//! Boîte de dialogue « enregistrer sous / ouvrir ».
//! Ordinateur : celle du système (rfd). Android : pas encore disponible, le choix est
//! considéré comme annulé (l'échange de fichiers avec le PC viendra avec la synchronisation).

#[cfg(desktop)]
pub use rfd::FileDialog;

#[cfg(mobile)]
#[derive(Default)]
pub struct FileDialog;

#[cfg(mobile)]
impl FileDialog {
    pub fn new() -> Self {
        FileDialog
    }
    pub fn set_title(self, _: impl Into<String>) -> Self {
        self
    }
    pub fn set_file_name(self, _: impl Into<String>) -> Self {
        self
    }
    pub fn add_filter(self, _: impl Into<String>, _: &[impl ToString]) -> Self {
        self
    }
    pub fn save_file(self) -> Option<std::path::PathBuf> {
        None
    }
    pub fn pick_file(self) -> Option<std::path::PathBuf> {
        None
    }
}
