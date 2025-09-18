use leptos::*;

use super::GlobalStateForLeptos;
use crate::log;

#[derive(Clone, Debug, Default)]
pub struct InputGroupState {
    pub open_dialog: RwSignal<OpenDialogComponent>,
    /// this prop enables switching between compact and full input group in mobile mode (InputGroupMobile component).
    pub show_full_input: RwSignal<bool>,
}

impl GlobalStateForLeptos for InputGroupState {}

impl InputGroupState {
    pub fn set_open_dialog(dialog: OpenDialogComponent) {
        // log!("Setting dialog to: {:?}", dialog);
        Self::get().open_dialog.update(|d| *d = dialog);
    }

    pub fn set_destination_open() {
        // log!("Opening destination dialog");
        Self::set_open_dialog(OpenDialogComponent::CityListComponent);
        // Self::get().open_dialog.update(|d| *d = OpenDialogComponent::CityListComponent);
    }
    pub fn set_close_dialog() {
        // log!("Closing dialog");
        Self::set_open_dialog(OpenDialogComponent::None);
    }

    pub fn set_date_open() {
        // log!("Opening date dialog");
        Self::set_open_dialog(OpenDialogComponent::DateComponent);
    }

    pub fn set_guest_open() {
        // log!("Opening guest dialog");
        Self::set_open_dialog(OpenDialogComponent::GuestComponent);
    }

    pub fn is_destination_open() -> bool {
        Self::get().open_dialog.get().is_destination_open()
        // log!("Checking if destination is open: {}", is_open);
    }

    pub fn is_date_open() -> bool {
        Self::get().open_dialog.get().is_date_open()
        // log!("Checking if date is open: {}", is_open);
    }

    pub fn is_guest_open() -> bool {
        Self::get().open_dialog.get().is_guest_open()
        // log!("Checking if guest is open: {}", is_open);
    }

    pub fn toggle_dialog(dialog: OpenDialogComponent) {
        let current = Self::get().open_dialog.get_untracked();
        if current.matches(dialog) {
            Self::set_open_dialog(OpenDialogComponent::None);
        } else {
            Self::set_open_dialog(dialog);
        }
    }

    pub fn toggle_show_full_input() {
        let current = Self::get().show_full_input.get_untracked();
        Self::get().show_full_input.update(|d| *d = !current);
    }

    pub fn is_open_show_full_input() -> bool {
        Self::get().show_full_input.get()
        // log!("Checking if show_full_input is open: {}", is_open);
    }

    pub fn set_show_full_input(show_full_input: bool) {
        Self::get().show_full_input.update(|d| *d = show_full_input);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum OpenDialogComponent {
    CityListComponent,
    DateComponent,
    GuestComponent,
    #[default]
    None,
}

impl OpenDialogComponent {
    pub fn matches(&self, other: OpenDialogComponent) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(&other)
    }

    pub fn is_destination_open(&self) -> bool {
        matches!(self, OpenDialogComponent::CityListComponent)
    }

    pub fn is_date_open(&self) -> bool {
        matches!(self, OpenDialogComponent::DateComponent)
    }

    pub fn is_guest_open(&self) -> bool {
        matches!(self, OpenDialogComponent::GuestComponent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches() {
        let city_dialog = OpenDialogComponent::CityListComponent;
        let date_dialog = OpenDialogComponent::DateComponent;
        let guest_dialog = OpenDialogComponent::GuestComponent;
        let none_dialog = OpenDialogComponent::None;

        // Test matching with same component
        assert!(city_dialog.matches(OpenDialogComponent::CityListComponent));
        assert!(date_dialog.matches(OpenDialogComponent::DateComponent));
        assert!(guest_dialog.matches(OpenDialogComponent::GuestComponent));
        assert!(none_dialog.matches(OpenDialogComponent::None));

        // Test non-matching cases
        assert!(!city_dialog.matches(OpenDialogComponent::DateComponent));
        assert!(!date_dialog.matches(OpenDialogComponent::GuestComponent));
        assert!(!guest_dialog.matches(OpenDialogComponent::None));
        assert!(!none_dialog.matches(OpenDialogComponent::CityListComponent));
    }

    #[test]
    fn test_is_destination_open() {
        assert!(OpenDialogComponent::CityListComponent.is_destination_open());
        assert!(!OpenDialogComponent::DateComponent.is_destination_open());
        assert!(!OpenDialogComponent::GuestComponent.is_destination_open());
        assert!(!OpenDialogComponent::None.is_destination_open());
    }

    #[test]
    fn test_is_date_open() {
        assert!(!OpenDialogComponent::CityListComponent.is_date_open());
        assert!(OpenDialogComponent::DateComponent.is_date_open());
        assert!(!OpenDialogComponent::GuestComponent.is_date_open());
        assert!(!OpenDialogComponent::None.is_date_open());
    }

    #[test]
    fn test_is_guest_open() {
        assert!(!OpenDialogComponent::CityListComponent.is_guest_open());
        assert!(!OpenDialogComponent::DateComponent.is_guest_open());
        assert!(OpenDialogComponent::GuestComponent.is_guest_open());
        assert!(!OpenDialogComponent::None.is_guest_open());
    }

    #[test]
    fn test_default() {
        assert!(matches!(
            OpenDialogComponent::default(),
            OpenDialogComponent::None
        ));
    }
}
