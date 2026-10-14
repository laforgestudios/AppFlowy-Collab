use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use yrs::{Any, encoding::serde::from_any};

use super::{LayoutSetting, LayoutSettingBuilder};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarLayoutSetting {
  #[serde(default)]
  pub layout_ty: CalendarLayout,
  #[serde(default)]
  pub first_day_of_week: i32,
  #[serde(default)]
  pub show_weekends: bool,
  #[serde(default)]
  pub show_week_numbers: bool,
  #[serde(default)]
  pub field_id: String,
}

impl From<LayoutSetting> for CalendarLayoutSetting {
  fn from(setting: LayoutSetting) -> Self {
    from_any(&Any::from(setting)).unwrap()
  }
}

impl From<CalendarLayoutSetting> for LayoutSetting {
  fn from(setting: CalendarLayoutSetting) -> Self {
    LayoutSettingBuilder::from([
      ("layout_ty".into(), Any::BigInt(setting.layout_ty.value())),
      (
        "first_day_of_week".into(),
        Any::BigInt(setting.first_day_of_week as i64),
      ),
      (
        "show_week_numbers".into(),
        Any::Bool(setting.show_week_numbers),
      ),
      ("show_weekends".into(), Any::Bool(setting.show_weekends)),
      ("field_id".into(), setting.field_id.into()),
    ])
  }
}

impl CalendarLayoutSetting {
  pub fn new(field_id: String) -> Self {
    CalendarLayoutSetting {
      layout_ty: CalendarLayout::default(),
      first_day_of_week: DEFAULT_FIRST_DAY_OF_WEEK,
      show_weekends: DEFAULT_SHOW_WEEKENDS,
      show_week_numbers: DEFAULT_SHOW_WEEK_NUMBERS,
      field_id,
    }
  }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum CalendarLayout {
  #[default]
  Month = 0,
  Week = 1,
  Day = 2,
}

impl From<i64> for CalendarLayout {
  fn from(value: i64) -> Self {
    match value {
      0 => CalendarLayout::Month,
      1 => CalendarLayout::Week,
      2 => CalendarLayout::Day,
      _ => CalendarLayout::Month,
    }
  }
}

impl CalendarLayout {
  pub fn value(&self) -> i64 {
    *self as i64
  }
}

pub const DEFAULT_FIRST_DAY_OF_WEEK: i32 = 0;
pub const DEFAULT_SHOW_WEEKENDS: bool = true;
pub const DEFAULT_SHOW_WEEK_NUMBERS: bool = true;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BoardLayoutSetting {
  #[serde(default)]
  pub hide_ungrouped_column: bool,
  #[serde(default)]
  pub collapse_hidden_groups: bool,
}

impl BoardLayoutSetting {
  pub fn new() -> Self {
    Self::default()
  }
}

impl From<LayoutSetting> for BoardLayoutSetting {
  fn from(setting: LayoutSetting) -> Self {
    from_any(&Any::from(setting)).unwrap()
  }
}

impl From<BoardLayoutSetting> for LayoutSetting {
  fn from(setting: BoardLayoutSetting) -> Self {
    LayoutSettingBuilder::from([
      (
        "hide_ungrouped_column".into(),
        setting.hide_ungrouped_column.into(),
      ),
      (
        "collapse_hidden_groups".into(),
        setting.collapse_hidden_groups.into(),
      ),
    ])
  }
}

/// Layout setting of a timeline view. Stored under the [DatabaseLayout::Timeline] key with
/// the same `field_id`/`layout_ty` keys as [CalendarLayoutSetting]; clients may add more keys
/// (end date, dependencies, progress...) which are left untouched.
///
/// [DatabaseLayout::Timeline]: super::DatabaseLayout::Timeline
#[derive(Debug, Clone)]
pub struct TimelineLayoutSetting {
  pub field_id: String,
  pub layout_ty: TimelineLayout,
  pub show_table: bool,
}

impl TimelineLayoutSetting {
  pub fn new(field_id: String) -> Self {
    TimelineLayoutSetting {
      field_id,
      layout_ty: TimelineLayout::default(),
      show_table: DEFAULT_TIMELINE_SHOW_TABLE,
    }
  }
}

impl From<LayoutSetting> for TimelineLayoutSetting {
  fn from(setting: LayoutSetting) -> Self {
    let field_id = match setting.get("field_id") {
      Some(Any::String(s)) => s.to_string(),
      _ => String::new(),
    };
    let layout_ty = match setting.get("layout_ty") {
      Some(Any::BigInt(v)) => TimelineLayout::from(*v),
      Some(Any::Number(v)) => TimelineLayout::from(*v as i64),
      _ => TimelineLayout::default(),
    };
    let show_table = match setting.get("show_table") {
      Some(Any::Bool(v)) => *v,
      _ => DEFAULT_TIMELINE_SHOW_TABLE,
    };
    TimelineLayoutSetting {
      field_id,
      layout_ty,
      show_table,
    }
  }
}

impl From<TimelineLayoutSetting> for LayoutSetting {
  fn from(setting: TimelineLayoutSetting) -> Self {
    LayoutSettingBuilder::from([
      ("field_id".into(), setting.field_id.into()),
      ("layout_ty".into(), Any::BigInt(setting.layout_ty.value())),
      ("show_table".into(), Any::Bool(setting.show_table)),
    ])
  }
}

/// Timeline scale, finest to coarsest.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum TimelineLayout {
  Hours = 0,
  Day = 1,
  Week = 2,
  BiWeek = 3,
  #[default]
  Month = 4,
  Quarter = 5,
  Year = 6,
}

impl From<i64> for TimelineLayout {
  fn from(value: i64) -> Self {
    match value {
      0 => TimelineLayout::Hours,
      1 => TimelineLayout::Day,
      2 => TimelineLayout::Week,
      3 => TimelineLayout::BiWeek,
      4 => TimelineLayout::Month,
      5 => TimelineLayout::Quarter,
      6 => TimelineLayout::Year,
      _ => TimelineLayout::default(),
    }
  }
}

impl TimelineLayout {
  pub fn value(&self) -> i64 {
    *self as i64
  }
}

pub const DEFAULT_TIMELINE_SHOW_TABLE: bool = true;
