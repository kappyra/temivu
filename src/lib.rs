//! An experimental, incomplete egui port of selected Forui design concepts.
//! See the project coverage matrix for parity and verification status.

#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

pub mod accordion;
pub mod alert;
pub mod autocomplete;
pub mod avatar;
pub mod badge;
pub mod bottom_navigation;
pub mod breadcrumb;
pub mod button;
pub mod calendar;
pub mod card;
pub mod checkbox;
pub mod context_menu;
pub mod date_field;
pub mod date_time_picker;
pub mod dialog;
pub mod divider;
mod generated_localizations;
mod generated_palette;
pub mod header;
pub mod item;
pub mod label;
pub mod line_calendar;
pub mod localization;
pub mod motion;
pub mod multi_select;
pub mod otp_field;
pub mod pagination;
pub mod picker;
pub mod popover;
pub mod popover_menu;
pub mod progress;
pub mod radio;
pub mod range_slider;
pub mod resizable;
pub mod scaffold;
pub mod select;
pub mod select_group;
pub mod select_menu_tile;
pub mod select_tile_group;
pub mod sheet;
pub mod sidebar;
pub mod slider;
pub mod switch;
pub mod tabs;
pub mod text_field;
pub mod theme;
pub mod tile;
pub mod time_field;
pub mod time_picker;
pub mod toast;
pub mod toaster;
pub mod tokens;
pub mod tooltip;

pub use accordion::{AccordionItem, AccordionState};
pub use alert::{Alert, AlertStyle, AlertVariant};
pub use autocomplete::{
    Autocomplete, AutocompleteFilter, AutocompleteOption, AutocompleteResponse,
};
pub use avatar::Avatar;
pub use badge::{Badge, BadgeStyle, BadgeVariant};
pub use bottom_navigation::{NavigationItem, bottom_navigation};
pub use breadcrumb::{Breadcrumb, BreadcrumbItem};
pub use button::{Button, ButtonSize, ButtonVariant, InputMode};
pub use calendar::{
    Calendar, CalendarLabels, CalendarResponse, CalendarState, Date, DateError, Weekday, month_name,
};
pub use card::{Card, CardStyle};
pub use checkbox::Checkbox;
pub use context_menu::context_menu;
pub use date_field::{
    DateField, DateFieldLabels, DateFieldResponse, DateFieldState, DateFormatter, DateParseError,
    DateParser, format_iso_date, parse_iso_date,
};
pub use date_time_picker::{
    DateTime, DateTimePicker, DateTimePickerLabels, DateTimePickerResponse, DateTimePickerState,
};
pub use dialog::Dialog;
pub use divider::{Divider, DividerAxis, DividerStyle};
pub use header::Header;
pub use item::{Item, ItemVariant};
pub use label::{FieldLabel, LabelLayout};
pub use line_calendar::{
    LineCalendar, LineCalendarLabels, LineCalendarOrientation, LineCalendarResponse,
    LineCalendarState,
};
pub use localization::Messages;
pub use motion::{animate_bool, animate_f32};
pub use multi_select::MultiSelect;
pub use otp_field::{OtpField, OtpFieldResponse, OtpLabels};
pub use pagination::{
    PageItem, PaginationState, pagination, pagination_localized, pagination_localized_with_id,
    pagination_with_id,
};
pub use picker::{Picker, PickerItem, PickerResponse};
pub use popover::Popover;
pub use popover_menu::{MenuItems, PopoverMenu};
pub use progress::{CircularProgress, CircularSize, Motion, Progress};
pub use radio::Radio;
pub use range_slider::RangeSlider;
pub use resizable::{Resizable, ResizableAxis, ResizableResponse};
pub use scaffold::Scaffold;
pub use select::{Select, SelectOption};
pub use select_group::{SelectGroup, SelectGroupResponse, inherited_select_error};
pub use select_menu_tile::SelectMenuTile;
pub use select_tile_group::{SelectTile, SelectTileGroup, SelectTileGroupResponse};
pub use sheet::Sheet;
pub use sidebar::{Sidebar, SidebarGroup, SidebarItem, SidebarResponse, SidebarState};
pub use slider::{Slider, SliderMark};
pub use switch::Switch;
pub use tabs::Tabs;
pub use text_field::{TextField, TextFieldOutput, Validator};
pub use theme::{Easing, FontWeight, MotionSettings, TextStyleToken, Theme, Typography};
pub use tile::Tile;
pub use time_field::{TimeField, TimeFieldResponse, TimeFieldState};
pub use time_picker::{
    Time, TimeError, TimeParseError, TimePicker, TimePickerLabels, TimePickerResponse,
};
pub use toast::{Toast, ToastState, ToastVariant};
pub use toaster::{ToastEntry, Toaster, ToasterResponse, ToasterState};
pub use tokens::{Breakpoints, FieldSizes, Sizes, ViewportClass};
