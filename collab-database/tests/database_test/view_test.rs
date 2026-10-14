use std::sync::Arc;

use assert_json_diff::assert_json_eq;
use collab::core::collab::{CollabOptions, default_client_id};
use collab::core::origin::CollabOrigin;
use collab::preclude::{Any, Collab};
use collab::util::AnyMapExt;
use collab_database::database::{DatabaseBody, DatabaseData, gen_row_id};
use collab_database::entity::CreateViewParams;
use collab_database::fields::Field;
use collab_database::rows::{CreateRowParams, Row};
use collab_database::views::{
  DatabaseLayout, LayoutSettingBuilder, OrderObjectPosition, TimelineLayout, TimelineLayoutSetting,
};
use futures::StreamExt;
use nanoid::nanoid;

use crate::database_test::helper::{
  create_database, create_database_with_default_data, default_field_settings_by_layout,
};
use crate::helper::TestFilter;

#[tokio::test]
async fn create_initial_database_test() {
  let database_id = uuid::Uuid::new_v4().to_string();
  let database_test = create_database(1, &database_id);

  let all_rows: Vec<Row> = database_test
    .get_all_rows(20, None, false)
    .await
    .filter_map(|result| async move { result.ok() })
    .collect()
    .await;
  assert_eq!(database_test.get_all_field_orders().len(), 0);
  assert_eq!(all_rows.len(), 0);
  assert_eq!(database_test.get_database_id(), database_id);

  let views = database_test.get_all_views();
  assert_eq!(views.len(), 1);
  assert_eq!(views[0].database_id, database_id);
  assert_ne!(views[0].database_id, views[0].id);
  assert_eq!(views[0].name, "my first database view".to_string());

  let encoded_collab = database_test
    .encode_collab_v1(|_| Ok::<_, anyhow::Error>(()))
    .unwrap();
  let options =
    CollabOptions::new("".to_string(), default_client_id()).with_data_source(encoded_collab.into());
  let collab = Collab::new_with_options(CollabOrigin::Empty, options).unwrap();
  let database_id_from_collab = DatabaseBody::database_id_from_collab(&collab).unwrap();
  assert_eq!(database_id_from_collab, database_id);
}

#[tokio::test]
async fn create_database_with_single_view_test() {
  let database_id = uuid::Uuid::new_v4();
  let database_test = create_database_with_default_data(1, &database_id.to_string()).await;
  let view = database_test.get_view("v1").unwrap();
  assert_eq!(view.row_orders.len(), 3);
  assert_eq!(view.field_orders.len(), 3);
}

#[tokio::test]
async fn get_database_views_meta_test() {
  let database_id = uuid::Uuid::new_v4();
  let database_test = create_database_with_default_data(1, &database_id.to_string()).await;
  let views = database_test.get_all_database_views_meta();
  assert_eq!(views.len(), 1);
  let view = database_test.get_view("v1").unwrap();
  assert_eq!(view.name, "my first database view");
}

#[tokio::test]
async fn create_same_database_view_twice_test() {
  let database_id = uuid::Uuid::new_v4();
  let mut database_test = create_database_with_default_data(1, &database_id.to_string()).await;
  let params = CreateViewParams {
    database_id: database_id.to_string(),
    view_id: "v1".to_string(),
    name: "my second grid".to_string(),
    layout: DatabaseLayout::Grid,
    ..Default::default()
  };
  database_test.create_linked_view(params).unwrap();
  let view = database_test.get_view("v1").unwrap();

  assert_eq!(view.name, "my second grid");
}

#[tokio::test]
async fn create_database_row_test() {
  let database_id = uuid::Uuid::new_v4().to_string();
  let mut database_test = create_database_with_default_data(1, &database_id).await;
  let row_id = gen_row_id();
  database_test
    .create_row(CreateRowParams::new(row_id.clone(), database_id.clone()))
    .await
    .unwrap();

  let view = database_test.get_view("v1").unwrap();
  assert_json_eq!(view.row_orders.last().unwrap().id, row_id);
}

#[tokio::test]
async fn create_database_field_test() {
  let database_id = uuid::Uuid::new_v4().to_string();
  let mut database_test = create_database_with_default_data(1, &database_id).await;

  let field_id = nanoid!(4);
  database_test.create_field(
    None,
    Field {
      id: field_id.clone(),
      name: "my third field".to_string(),
      ..Default::default()
    },
    &OrderObjectPosition::default(),
    default_field_settings_by_layout(),
  );

  let view = database_test.get_view("v1").unwrap();
  assert_json_eq!(view.field_orders.last().unwrap().id, field_id);
}

#[tokio::test]
async fn create_database_view_with_filter_test() {
  let database_id = uuid::Uuid::new_v4();
  let mut database_test = create_database_with_default_data(1, &database_id.to_string()).await;
  let filter_1 = TestFilter {
    id: "filter1".to_string(),
    field_id: "".to_string(),
    field_type: Default::default(),
    condition: 0,
    content: "".to_string(),
  };

  let filter_2 = TestFilter {
    id: "filter2".to_string(),
    field_id: "".to_string(),
    field_type: Default::default(),
    condition: 0,
    content: "".to_string(),
  };

  let params = CreateViewParams {
    database_id: database_id.to_string(),
    view_id: "v1".to_string(),
    name: "my first grid".to_string(),
    filters: vec![filter_1.into(), filter_2.into()],
    layout: DatabaseLayout::Grid,
    ..Default::default()
  };
  database_test.create_linked_view(params).unwrap();

  let view = database_test.get_view("v1").unwrap();
  let filters = view
    .filters
    .into_iter()
    .map(|value| TestFilter::try_from(value).unwrap())
    .collect::<Vec<TestFilter>>();
  assert_eq!(filters.len(), 2);
  assert_eq!(filters[0].id, "filter1");
  assert_eq!(filters[1].id, "filter2");
}

#[tokio::test]
async fn create_database_view_with_layout_setting_test() {
  let database_id = uuid::Uuid::new_v4();
  let mut database_test = create_database_with_default_data(1, &database_id.to_string()).await;
  let grid_setting =
    LayoutSettingBuilder::from([("1".into(), 123.into()), ("2".into(), "abc".into())]);

  let params = CreateViewParams {
    database_id: database_id.to_string(),
    view_id: "v1".to_string(),
    name: "my first grid".to_string(),
    layout: DatabaseLayout::Grid,
    ..Default::default()
  }
  .with_layout_setting(grid_setting);
  database_test.create_linked_view(params).unwrap();

  let view = database_test.get_view("v1").unwrap();
  let grid_layout_setting = view.layout_settings.get(&DatabaseLayout::Grid).unwrap();
  assert_eq!(grid_layout_setting.get_as::<i64>("1").unwrap(), 123);
  assert_eq!(
    grid_layout_setting.get("2").unwrap(),
    &Any::String(Arc::from("abc".to_string()))
  );
}

#[tokio::test]
async fn delete_database_view_test() {
  let database_id = uuid::Uuid::new_v4();
  let mut database_test = create_database_with_default_data(1, &database_id.to_string()).await;
  for i in 2..5 {
    let params = CreateViewParams {
      database_id: database_id.to_string(),
      view_id: format!("v{}", i),
      ..Default::default()
    };
    database_test.create_linked_view(params).unwrap();
  }

  let views = database_test.get_all_views();
  assert_eq!(views.len(), 4);

  let deleted_view_id = "v3".to_string();
  database_test.delete_view(&deleted_view_id);
  let views = database_test
    .get_all_views()
    .iter()
    .map(|view| view.id.clone())
    .collect::<Vec<String>>();
  assert_eq!(views.len(), 3);
  assert!(!views.contains(&deleted_view_id));
}

#[tokio::test]
async fn duplicate_database_view_test() {
  let database_id = uuid::Uuid::new_v4();
  let mut database_test = create_database_with_default_data(1, &database_id.to_string()).await;

  let views = database_test.get_all_views();
  assert_eq!(views.len(), 1);

  let view = database_test.get_view("v1").unwrap();
  let duplicated_view = database_test.duplicate_linked_view("v1").unwrap();

  let views = database_test.get_all_views();
  assert_eq!(views.len(), 2);

  assert_eq!(duplicated_view.name, format!("{}-copy", view.name));
  assert_ne!(view.id, duplicated_view.id);
  assert_eq!(view.database_id, duplicated_view.database_id);
}

#[tokio::test]
async fn database_data_serde_test() {
  let database_id = uuid::Uuid::new_v4();
  let database_test = create_database_with_default_data(1, &database_id.to_string()).await;
  let database_data = database_test.get_database_data(20, false).await;

  let json = database_data.to_json().unwrap();
  let database_data2 = DatabaseData::from_json(&json).unwrap();
  assert_eq!(database_data.fields.len(), database_data2.fields.len());
  assert_eq!(database_data.rows.len(), database_data2.rows.len());
}

#[tokio::test]
async fn get_database_view_layout_test() {
  let database_id = uuid::Uuid::new_v4();
  let database_test = create_database_with_default_data(1, &database_id.to_string()).await;

  let layout = database_test.get_database_view_layout("v1");
  assert_eq!(layout, DatabaseLayout::Grid);
}

#[tokio::test]
async fn update_database_view_layout_test() {
  let database_id = uuid::Uuid::new_v4();
  let mut database_test = create_database_with_default_data(1, &database_id.to_string()).await;
  database_test.update_database_view("v1", |update| {
    update.set_layout_type(DatabaseLayout::Calendar);
  });

  let layout = database_test.get_database_view_layout("v1");
  assert_eq!(layout, DatabaseLayout::Calendar);
}

#[tokio::test]
async fn database_view_layout_round_trip_test() {
  use std::str::FromStr;

  let layouts = [
    (DatabaseLayout::Grid, 0),
    (DatabaseLayout::Board, 1),
    (DatabaseLayout::Calendar, 2),
    (DatabaseLayout::Chart, 3),
    (DatabaseLayout::List, 4),
    (DatabaseLayout::Gallery, 5),
    (DatabaseLayout::Feed, 6),
    (DatabaseLayout::Form, 7),
    (DatabaseLayout::Timeline, 8),
  ];

  let database_id = uuid::Uuid::new_v4();
  let mut database_test = create_database_with_default_data(1, &database_id.to_string()).await;

  for (layout, value) in layouts {
    assert_eq!(layout as i64, value);
    assert_eq!(DatabaseLayout::from(value), layout);
    assert_eq!(layout.as_ref(), value.to_string());
    assert_eq!(
      DatabaseLayout::from_str(&value.to_string()).unwrap(),
      layout
    );
    assert_eq!(serde_json::to_string(&layout).unwrap(), value.to_string());
    assert_eq!(
      layout.requires_date_field(),
      matches!(layout, DatabaseLayout::Calendar | DatabaseLayout::Timeline)
    );

    // Layout type of an existing view.
    database_test.update_database_view("v1", |update| {
      update.set_layout_type(layout);
    });
    assert_eq!(database_test.get_database_view_layout("v1"), layout);

    // A linked view created with this layout keeps its layout and its layout setting.
    let view_id = format!("layout-{}", value);
    let params = CreateViewParams::new(
      database_id.to_string(),
      view_id.clone(),
      view_id.clone(),
      layout,
    )
    .with_layout_setting(LayoutSettingBuilder::from([("k".into(), value.into())]));
    database_test.create_linked_view(params).unwrap();
    let view = database_test.get_view(&view_id).unwrap();
    assert_eq!(view.layout, layout);
    assert_eq!(
      view
        .layout_settings
        .get(&layout)
        .unwrap()
        .get_as::<i64>("k")
        .unwrap(),
      value
    );
  }

  // Timeline layout setting survives a round trip through the view.
  let params = CreateViewParams::new(
    database_id.to_string(),
    "timeline".to_string(),
    "timeline".to_string(),
    DatabaseLayout::Timeline,
  )
  .with_layout_setting(TimelineLayoutSetting::new("date_field".to_string()).into());
  database_test.create_linked_view(params).unwrap();
  let view = database_test.get_view("timeline").unwrap();
  let setting = TimelineLayoutSetting::from(
    view
      .layout_settings
      .get(&DatabaseLayout::Timeline)
      .unwrap()
      .clone(),
  );
  assert_eq!(setting.field_id, "date_field");
  assert_eq!(setting.layout_ty, TimelineLayout::Month);
  assert!(setting.show_table);

  // Unknown values still fall back to Grid.
  assert_eq!(DatabaseLayout::from(9), DatabaseLayout::Grid);
  assert!(DatabaseLayout::from_str("9").is_err());
}

#[tokio::test]
async fn validate_database_test() {
  let database_id = uuid::Uuid::new_v4();
  let database_test = create_database_with_default_data(1, &database_id.to_string()).await;
  assert!(database_test.database.validate().is_ok())
}
