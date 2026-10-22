use crate::notion_api::{
    BASE_URL, NotionApi,
    common::RequestStatus,
    types::page::{Page, PagePropertyDesc, PageTitle},
};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct DatasourceQueryRes {
    #[serde(rename = "type")]
    pub response_type: String,
    pub object: String,
    pub next_cursor: Option<String>,
    pub has_more: bool,
    pub results: Vec<Page>,
    pub request_status: Option<RequestStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSource {
    pub id: String,
    #[serde(default)]
    pub title: Vec<PageTitle>,
    #[serde(default)]
    pub properties: HashMap<String, PagePropertyDesc>,
}

// ---------------------------------------------------------------------------
// Small building blocks
// ---------------------------------------------------------------------------

/// Marker that serializes as the JSON literal `true`.
/// Notion's `is_empty` / `is_not_empty` conditions only accept `true`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct True;

impl Serialize for True {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bool(true)
    }
}

impl<'de> Deserialize<'de> for True {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        if bool::deserialize(d)? {
            Ok(True)
        } else {
            Err(D::Error::custom("expected `true`"))
        }
    }
}

/// Serializes as `{}` (used by `this_week`, `past_month`, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EmptyObject {}

/// Either a single string or an array of strings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrStringArray {
    One(String),
    Many(Vec<String>),
}

impl From<&str> for StringOrStringArray {
    fn from(s: &str) -> Self {
        Self::One(s.to_owned())
    }
}
impl From<String> for StringOrStringArray {
    fn from(s: String) -> Self {
        Self::One(s)
    }
}
impl From<Vec<String>> for StringOrStringArray {
    fn from(v: Vec<String>) -> Self {
        Self::Many(v)
    }
}

/// A user id, or the literal `"me"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum PersonRef {
    Me,
    Id(String),
}

impl From<String> for PersonRef {
    fn from(s: String) -> Self {
        if s == "me" { Self::Me } else { Self::Id(s) }
    }
}
impl From<PersonRef> for String {
    fn from(p: PersonRef) -> Self {
        match p {
            PersonRef::Me => "me".to_owned(),
            PersonRef::Id(id) => id,
        }
    }
}

/// Named relative dates accepted by date conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelativeDate {
    Today,
    Tomorrow,
    Yesterday,
    OneWeekAgo,
    OneWeekFromNow,
    OneMonthAgo,
    OneMonthFromNow,
}

/// An ISO 8601 date string (`"2026-10-22"`) or a [`RelativeDate`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DateOrRelativeDate {
    Relative(RelativeDate),
    Date(String),
}

impl From<RelativeDate> for DateOrRelativeDate {
    fn from(r: RelativeDate) -> Self {
        Self::Relative(r)
    }
}
impl From<&str> for DateOrRelativeDate {
    fn from(s: &str) -> Self {
        Self::Date(s.to_owned())
    }
}
impl From<String> for DateOrRelativeDate {
    fn from(s: String) -> Self {
        Self::Date(s)
    }
}

// ---------------------------------------------------------------------------
// Per-type conditions
// ---------------------------------------------------------------------------

/// `is_empty` / `is_not_empty` only (used by `files`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExistenceCondition {
    IsEmpty(True),
    IsNotEmpty(True),
}

/// Conditions for title, rich_text, url, email, phone_number (and formula strings).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextCondition {
    Equals(String),
    DoesNotEqual(String),
    Contains(String),
    DoesNotContain(String),
    StartsWith(String),
    EndsWith(String),
    IsEmpty(True),
    IsNotEmpty(True),
}

/// Conditions for number, unique_id (and formula numbers).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumberCondition {
    Equals(f64),
    DoesNotEqual(f64),
    GreaterThan(f64),
    LessThan(f64),
    GreaterThanOrEqualTo(f64),
    LessThanOrEqualTo(f64),
    IsEmpty(True),
    IsNotEmpty(True),
}

/// Conditions for checkbox (and formula checkboxes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckboxCondition {
    Equals(bool),
    DoesNotEqual(bool),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectCondition {
    Equals(StringOrStringArray),
    DoesNotEqual(StringOrStringArray),
    IsEmpty(True),
    IsNotEmpty(True),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MultiSelectCondition {
    Contains(StringOrStringArray),
    DoesNotContain(StringOrStringArray),
    IsEmpty(True),
    IsNotEmpty(True),
}

/// `group_equals` / `group_does_not_equal` match status *groups* rather than
/// options and require the `Notion-Beta: status-group-filters-2026-10-06` header.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusCondition {
    Equals(StringOrStringArray),
    DoesNotEqual(StringOrStringArray),
    GroupEquals(StringOrStringArray),
    GroupDoesNotEqual(StringOrStringArray),
    IsEmpty(True),
    IsNotEmpty(True),
}

/// Conditions for date, created_time, last_edited_time (and formula dates).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DateCondition {
    Equals(DateOrRelativeDate),
    Before(DateOrRelativeDate),
    After(DateOrRelativeDate),
    OnOrBefore(DateOrRelativeDate),
    OnOrAfter(DateOrRelativeDate),
    ThisWeek(EmptyObject),
    PastWeek(EmptyObject),
    PastMonth(EmptyObject),
    PastYear(EmptyObject),
    NextWeek(EmptyObject),
    NextMonth(EmptyObject),
    NextYear(EmptyObject),
    IsEmpty(True),
    IsNotEmpty(True),
}

/// Conditions for people, created_by, last_edited_by.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeopleCondition {
    Contains(PersonRef),
    DoesNotContain(PersonRef),
    IsEmpty(True),
    IsNotEmpty(True),
}

/// Values are page ids.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationCondition {
    Contains(String),
    DoesNotContain(String),
    IsEmpty(True),
    IsNotEmpty(True),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Verified,
    Expired,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationCondition {
    Status(VerificationStatus),
    DoesNotEqual(VerificationStatus),
}

/// `{ "formula": { "string": {...} } }` etc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormulaCondition {
    String(TextCondition),
    Checkbox(CheckboxCondition),
    Number(NumberCondition),
    Date(DateCondition),
}

/// Subfilter applied to the rolled-up values (`any` / `none` / `every`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollupSubfilter {
    RichText(TextCondition),
    Number(NumberCondition),
    Checkbox(CheckboxCondition),
    Select(SelectCondition),
    MultiSelect(MultiSelectCondition),
    Relation(RelationCondition),
    Date(DateCondition),
    People(PeopleCondition),
    Files(ExistenceCondition),
    Status(StatusCondition),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollupCondition {
    Any(RollupSubfilter),
    None(RollupSubfilter),
    Every(RollupSubfilter),
    Date(DateCondition),
    Number(NumberCondition),
}

// ---------------------------------------------------------------------------
// Property filter
// ---------------------------------------------------------------------------

/// The type-keyed part of a property filter: `{ "<type>": <condition> }`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyCondition {
    Title(TextCondition),
    RichText(TextCondition),
    Number(NumberCondition),
    Checkbox(CheckboxCondition),
    Select(SelectCondition),
    MultiSelect(MultiSelectCondition),
    Status(StatusCondition),
    Date(DateCondition),
    People(PeopleCondition),
    Files(ExistenceCondition),
    Url(TextCondition),
    Email(TextCondition),
    PhoneNumber(TextCondition),
    Relation(RelationCondition),
    CreatedBy(PeopleCondition),
    CreatedTime(DateCondition),
    LastEditedBy(PeopleCondition),
    LastEditedTime(DateCondition),
    Formula(FormulaCondition),
    UniqueId(NumberCondition),
    Rollup(RollupCondition),
    Verification(VerificationCondition),
}

/// `{ "property": "Status", "select": { "equals": "Done" } }`
///
/// The optional `type` discriminator Notion accepts is tolerated on input
/// (and retained in `declared_type`) but is not emitted on output, since the
/// type key itself already identifies the filter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyFilter {
    /// Property name or ID.
    pub property: String,
    #[serde(rename = "type", default, skip_serializing)]
    pub declared_type: Option<String>,
    #[serde(flatten)]
    pub condition: PropertyCondition,
}

impl PropertyFilter {
    pub fn new(property: impl Into<String>, condition: PropertyCondition) -> Self {
        Self {
            property: property.into(),
            declared_type: None,
            condition,
        }
    }
}

// ---------------------------------------------------------------------------
// Timestamp filter
// ---------------------------------------------------------------------------

/// `{ "timestamp": "created_time", "created_time": { "after": "2026-01-01" } }`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "timestamp", rename_all = "snake_case")]
pub enum TimestampFilter {
    CreatedTime { created_time: DateCondition },
    LastEditedTime { last_edited_time: DateCondition },
}

// ---------------------------------------------------------------------------
// Compound filters (and / or)
// ---------------------------------------------------------------------------

/// A leaf: either a property filter or a timestamp filter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyOrTimestampFilter {
    Property(PropertyFilter),
    Timestamp(TimestampFilter),
}

impl From<PropertyFilter> for PropertyOrTimestampFilter {
    fn from(f: PropertyFilter) -> Self {
        Self::Property(f)
    }
}
impl From<TimestampFilter> for PropertyOrTimestampFilter {
    fn from(f: TimestampFilter) -> Self {
        Self::Timestamp(f)
    }
}

/// Second (innermost) compound level: groups of leaf filters only. Max 100 items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeafGroup {
    Or(Vec<PropertyOrTimestampFilter>),
    And(Vec<PropertyOrTimestampFilter>),
}

/// An item inside a top-level group: a leaf filter or one nested group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GroupItem {
    Leaf(PropertyOrTimestampFilter),
    Group(LeafGroup),
}

impl From<PropertyFilter> for GroupItem {
    fn from(f: PropertyFilter) -> Self {
        Self::Leaf(f.into())
    }
}
impl From<TimestampFilter> for GroupItem {
    fn from(f: TimestampFilter) -> Self {
        Self::Leaf(f.into())
    }
}
impl From<PropertyOrTimestampFilter> for GroupItem {
    fn from(f: PropertyOrTimestampFilter) -> Self {
        Self::Leaf(f)
    }
}
impl From<LeafGroup> for GroupItem {
    fn from(g: LeafGroup) -> Self {
        Self::Group(g)
    }
}

/// Top-level compound filter. Max 100 items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopLevelGroup {
    Or(Vec<GroupItem>),
    And(Vec<GroupItem>),
}

/// The `filter` field of the request body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Filter {
    Group(TopLevelGroup),
    Property(PropertyFilter),
    Timestamp(TimestampFilter),
}

impl Filter {
    pub fn property(property: impl Into<String>, condition: PropertyCondition) -> Self {
        Self::Property(PropertyFilter::new(property, condition))
    }

    pub fn and<I, T>(items: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<GroupItem>,
    {
        Self::Group(TopLevelGroup::And(
            items.into_iter().map(Into::into).collect(),
        ))
    }

    pub fn or<I, T>(items: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<GroupItem>,
    {
        Self::Group(TopLevelGroup::Or(
            items.into_iter().map(Into::into).collect(),
        ))
    }
}

impl From<PropertyFilter> for Filter {
    fn from(f: PropertyFilter) -> Self {
        Self::Property(f)
    }
}
impl From<TimestampFilter> for Filter {
    fn from(f: TimestampFilter) -> Self {
        Self::Timestamp(f)
    }
}

// ---------------------------------------------------------------------------
// Sorts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortTimestamp {
    CreatedTime,
    LastEditedTime,
}

/// Earlier sorts in the request's `sorts` array take precedence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Sort {
    Property {
        property: String,
        direction: SortDirection,
    },
    Timestamp {
        timestamp: SortTimestamp,
        direction: SortDirection,
    },
}

impl Sort {
    pub fn property(property: impl Into<String>, direction: SortDirection) -> Self {
        Self::Property {
            property: property.into(),
            direction,
        }
    }
    pub fn timestamp(timestamp: SortTimestamp, direction: SortDirection) -> Self {
        Self::Timestamp {
            timestamp,
            direction,
        }
    }
}

// ---------------------------------------------------------------------------
// Request body
// ---------------------------------------------------------------------------

/// Restricts wiki results to pages or to child data sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultType {
    Page,
    DataSource,
}

/// JSON body for `POST /v1/data_sources/{id}/query`.
///
/// `filter_properties` is a *query-string* parameter, not part of this body.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct QueryDataSourceRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<Filter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sorts: Option<Vec<Sort>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_cursor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// `true` queries archived pages; omitted/`false` queries non-archived pages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_archived: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_type: Option<ResultType>,
}

impl NotionApi {
    pub async fn retrieve_datasource(&self, datasource_id: &Uuid) -> anyhow::Result<DataSource> {
        Ok(self
            .client
            .get(format!("{}data_sources/{}", BASE_URL, datasource_id))
            .send()
            .await?
            .error_for_status()?
            .json::<DataSource>()
            .await?)
    }

    pub async fn query_datasource(
        &self,
        datasource_id: &Uuid,
        filter_properties: &[String],
        query: QueryDataSourceRequest,
    ) -> anyhow::Result<DatasourceQueryRes> {
        let params = filter_properties
            .iter()
            .map(|prop| ("filter_properties[]", prop));

        let url = reqwest::Url::parse_with_params(
            &format!("{}data_sources/{}/query", BASE_URL, datasource_id),
            params,
        )?;

        Ok(self
            .client
            .post(url)
            .json(&query)
            .send()
            .await?
            .error_for_status()?
            .json::<DatasourceQueryRes>()
            .await?)
    }
}
