// models.rs
use chrono::{NaiveDate, NaiveDateTime, DateTime, Utc};
use diesel::prelude::*;
use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::Text;
use serde::{Deserialize, Serialize, Deserializer};
use std::io::Write;
use serde_json;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)]
#[diesel(sql_type = crate::schema::sql_types::TicketStatus)]
pub enum TicketStatus {
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "in-progress")]
    InProgress,
    #[serde(rename = "closed")]
    Closed,
}

impl ToSql<crate::schema::sql_types::TicketStatus, Pg> for TicketStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let s = match *self {
            TicketStatus::Open => "open",
            TicketStatus::InProgress => "in-progress",
            TicketStatus::Closed => "closed",
        };
        out.write_all(s.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::TicketStatus, Pg> for TicketStatus {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"open" => Ok(TicketStatus::Open),
            b"in-progress" => Ok(TicketStatus::InProgress),
            b"closed" => Ok(TicketStatus::Closed),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)]
#[diesel(sql_type = crate::schema::sql_types::TicketPriority)]
pub enum TicketPriority {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
}

impl ToSql<crate::schema::sql_types::TicketPriority, Pg> for TicketPriority {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let s = match *self {
            TicketPriority::Low => "low",
            TicketPriority::Medium => "medium",
            TicketPriority::High => "high",
        };
        out.write_all(s.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::TicketPriority, Pg> for TicketPriority {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"low" => Ok(TicketPriority::Low),
            b"medium" => Ok(TicketPriority::Medium),
            b"high" => Ok(TicketPriority::High),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Identifiable, Queryable)]
#[diesel(table_name = crate::schema::tickets)]
pub struct Ticket {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub status: TicketStatus,
    pub priority: TicketPriority,
    pub requester_uuid: Uuid,
    pub assignee_uuid: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub device_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::tickets)]
pub struct NewTicket {
    pub title: String,
    pub description: Option<String>,
    pub status: TicketStatus,
    pub priority: TicketPriority,
    pub requester_uuid: Uuid,
    pub assignee_uuid: Option<Uuid>,
    pub device_id: Option<i32>,
}

// Add a new struct for partial ticket updates
#[derive(Debug, Serialize, AsChangeset)]
#[diesel(table_name = crate::schema::tickets)]
pub struct TicketUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TicketStatus>,
    pub priority: Option<TicketPriority>,
    pub requester_uuid: Option<Uuid>,
    pub assignee_uuid: Option<Option<Uuid>>,
    pub updated_at: Option<NaiveDateTime>,
    pub device_id: Option<Option<i32>>,
}

// Custom implementation of Deserialize for TicketUpdate to handle ISO datetime strings
impl<'de> Deserialize<'de> for TicketUpdate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TicketUpdateHelper {
            title: Option<String>,
            description: Option<String>,
            status: Option<TicketStatus>,
            priority: Option<TicketPriority>,
            requester_uuid: Option<Uuid>,
            assignee_uuid: Option<Option<Uuid>>,
            #[serde(default)]
            updated_at: Option<String>,
            device_id: Option<Option<i32>>,
        }

        let helper = TicketUpdateHelper::deserialize(deserializer)?;
        
        // Parse the updated_at date if it exists
        let updated_at = if let Some(date_str) = helper.updated_at {
            // Try to parse the ISO datetime string
            match DateTime::parse_from_rfc3339(&date_str) {
                Ok(dt) => Some(dt.naive_utc()),
                Err(_) => {
                    // Fallback to other common formats if RFC3339 fails
                    match NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S") {
                        Ok(dt) => Some(dt),
                        Err(_) => {
                            // If all parsing attempts fail, use current time
                            println!("Warning: Could not parse date '{}', using current time", date_str);
                            Some(Utc::now().naive_utc())
                        }
                    }
                }
            }
        } else {
            None
        };

        Ok(TicketUpdate {
            title: helper.title,
            description: helper.description,
            status: helper.status,
            priority: helper.priority,
            requester_uuid: helper.requester_uuid,
            assignee_uuid: helper.assignee_uuid,
            updated_at,
            device_id: helper.device_id,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Identifiable, Queryable)]
#[diesel(table_name = crate::schema::devices)]
pub struct Device {
    pub id: i32,
    pub name: String,
    pub hostname: Option<String>,
    pub device_type: Option<String>,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub warranty_status: Option<String>,
    pub location: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub notes: Option<String>,
    pub user_id: Option<i32>,
    pub primary_user_uuid: Option<Uuid>,
    pub azure_device_id: Option<String>,
    pub intune_device_id: Option<String>,
    pub entra_device_id: Option<String>,
    pub compliance_state: Option<String>,
    pub last_sync_time: Option<NaiveDateTime>,
    pub operating_system: Option<String>,
    pub os_version: Option<String>,
    pub is_managed: Option<bool>,
    pub enrollment_date: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::devices)]
pub struct NewDevice {
    pub name: String,
    pub hostname: Option<String>,
    pub device_type: Option<String>,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub warranty_status: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub user_id: Option<i32>,
    pub primary_user_uuid: Option<Uuid>,
    pub azure_device_id: Option<String>,
    pub intune_device_id: Option<String>,
    pub entra_device_id: Option<String>,
    pub compliance_state: Option<String>,
    pub last_sync_time: Option<NaiveDateTime>,
    pub operating_system: Option<String>,
    pub os_version: Option<String>,
    pub is_managed: Option<bool>,
    pub enrollment_date: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::devices)]
pub struct DeviceUpdate {
    pub name: Option<String>,
    pub hostname: Option<String>,
    pub device_type: Option<String>,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub warranty_status: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub user_id: Option<i32>,
    pub primary_user_uuid: Option<Uuid>,
    pub azure_device_id: Option<String>,
    pub intune_device_id: Option<String>,
    pub entra_device_id: Option<String>,
    pub compliance_state: Option<String>,
    pub last_sync_time: Option<NaiveDateTime>,
    pub operating_system: Option<String>,
    pub os_version: Option<String>,
    pub is_managed: Option<bool>,
    pub enrollment_date: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Identifiable, Queryable, Associations)]
#[diesel(table_name = crate::schema::ticket_devices)]
#[diesel(belongs_to(Ticket))]
#[diesel(belongs_to(Device))]
#[diesel(primary_key(ticket_id, device_id))]
pub struct TicketDevice {
    pub ticket_id: i32,
    pub device_id: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::ticket_devices)]
pub struct NewTicketDevice {
    pub ticket_id: i32,
    pub device_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Identifiable, Queryable, Associations)]
#[diesel(table_name = crate::schema::comments)]
#[diesel(belongs_to(Ticket))]
pub struct Comment {
    pub id: i32,
    pub content: String,
    pub ticket_id: i32,
    pub user_id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::comments)]
pub struct NewComment {
    pub content: String,
    pub ticket_id: i32,
    pub user_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Identifiable, Associations, Clone)]
#[diesel(table_name = crate::schema::attachments)]
#[diesel(belongs_to(Comment))]
pub struct Attachment {
    pub id: i32,
    pub url: String,
    pub name: String,
    pub comment_id: Option<i32>,
}

impl Queryable<(
    diesel::sql_types::Integer,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Nullable<diesel::sql_types::Integer>,
), Pg> for Attachment {
    type Row = (
        i32,
        String,
        String,
        Option<i32>,
    );

    fn build(row: Self::Row) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Attachment {
            id: row.0,
            url: row.1,
            name: row.2,
            comment_id: row.3,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::attachments)]
pub struct NewAttachment {
    pub url: String,
    pub name: String,
    pub comment_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Identifiable, Associations)]
#[diesel(table_name = crate::schema::article_contents)]
#[diesel(belongs_to(Ticket))]
pub struct ArticleContent {
    pub id: i32,
    pub content: String,
    pub ticket_id: Option<i32>,
}

impl Queryable<(
    diesel::sql_types::Integer,
    diesel::sql_types::Text,
    diesel::sql_types::Nullable<diesel::sql_types::Integer>,
), Pg> for ArticleContent {
    type Row = (
        i32,
        String,
        Option<i32>,
    );

    fn build(row: Self::Row) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(ArticleContent {
            id: row.0,
            content: row.1,
            ticket_id: row.2,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::article_contents)]
pub struct NewArticleContent {
    pub content: String,
    pub ticket_id: i32,
}

// Composite struct for returning a complete ticket with all related data
#[derive(Debug, Serialize, Deserialize)]
pub struct CompleteTicket {
    #[serde(flatten)]
    pub ticket: Ticket,
    pub devices: Vec<Device>,
    pub comments: Vec<CommentWithAttachments>,
    pub article_content: Option<String>,
    pub linked_tickets: Vec<i32>,
    pub projects: Vec<Project>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentWithAttachments {
    #[serde(flatten)]
    pub comment: Comment,
    pub attachments: Vec<Attachment>,
    pub user: Option<UserInfo>,
}

// JSON import struct that matches the structure in tickets.json
#[derive(Debug, Serialize, Deserialize)]
pub struct TicketJson {
    pub id: i32,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub created: String,
    pub modified: String,
    pub assignee: String,
    pub requester: String,
    pub device: Option<DeviceJson>,
    pub comments: Option<Vec<CommentJson>>,
    pub article_content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceJson {
    pub id: String,
    pub name: String,
    pub hostname: String,
    #[serde(rename = "serialNumber")]
    pub serial_number: String,
    pub model: String,
    #[serde(rename = "warrantyStatus")]
    pub warranty_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentJson {
    pub id: i32,
    pub content: String,
    pub user_uuid: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub attachments: Vec<AttachmentJson>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttachmentJson {
    pub url: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketsJson {
    pub tickets: Vec<TicketJson>,
}

// Documentation Status Enum
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)]
#[diesel(sql_type = crate::schema::sql_types::DocumentationStatus)]
pub enum DocumentationStatus {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "published")]
    Published,
    #[serde(rename = "archived")]
    Archived,
}

impl ToSql<crate::schema::sql_types::DocumentationStatus, Pg> for DocumentationStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let s = match *self {
            DocumentationStatus::Draft => "draft",
            DocumentationStatus::Published => "published",
            DocumentationStatus::Archived => "archived",
        };
        out.write_all(s.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::DocumentationStatus, Pg> for DocumentationStatus {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"draft" => Ok(DocumentationStatus::Draft),
            b"published" => Ok(DocumentationStatus::Published),
            b"archived" => Ok(DocumentationStatus::Archived),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

// Documentation Page
#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Clone)]
#[diesel(table_name = crate::schema::documentation_pages)]
pub struct DocumentationPage {
    pub id: i32,
    pub uuid: Uuid,
    pub title: String,
    pub slug: Option<String>,
    pub icon: Option<String>,
    pub cover_image: Option<String>,
    pub status: DocumentationStatus,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub created_by: Uuid,
    pub last_edited_by: Uuid,
    pub parent_id: Option<i32>,
    pub ticket_id: Option<i32>,
    pub display_order: Option<i32>,
    pub is_public: bool,
    pub is_template: bool,
    pub archived_at: Option<chrono::NaiveDateTime>,
    pub yjs_state_vector: Option<Vec<u8>>,
    pub yjs_document: Option<Vec<u8>>,
    pub yjs_client_id: Option<i64>,
    pub estimated_reading_time: Option<i32>,
    pub word_count: Option<i32>,
    pub has_unsaved_changes: bool,
}

// Documentation Page with Children
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentationPageWithChildren {
    pub page: DocumentationPage,
    pub children: Vec<DocumentationPage>,
}

// Model for reordering pages
#[derive(Debug, Serialize, Deserialize)]
pub struct ReorderPagesRequest {
    pub parent_id: Option<i32>,
    pub page_orders: Vec<PageOrder>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageOrder {
    pub page_id: i32,
    pub display_order: i32,
}

// Model for moving a page to a new parent
#[derive(Debug, Serialize, Deserialize)]
pub struct MovePageRequest {
    pub page_id: i32,
    pub new_parent_id: Option<i32>,
    pub display_order: i32,
}

// User Role Enum
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)]
#[diesel(sql_type = crate::schema::sql_types::UserRole)]
pub enum UserRole {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "technician")]
    Technician,
    #[serde(rename = "user")]
    User,
}

impl ToSql<crate::schema::sql_types::UserRole, Pg> for UserRole {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let s = match *self {
            UserRole::Admin => "admin",
            UserRole::Technician => "technician",
            UserRole::User => "user",
        };
        out.write_all(s.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::UserRole, Pg> for UserRole {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"admin" => Ok(UserRole::Admin),
            b"technician" => Ok(UserRole::Technician),
            b"user" => Ok(UserRole::User),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

// User model - updated to match the actual database schema
#[derive(Debug, Serialize, Deserialize, Identifiable, Queryable)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: i32,
    pub uuid: Uuid,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub password_hash: Vec<u8>,
    pub pronouns: Option<String>,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub avatar_thumb: Option<String>,
    pub microsoft_uuid: Option<Uuid>,
}

// New user for creation
#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub uuid: Uuid,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub password_hash: Vec<u8>,
    pub pronouns: Option<String>,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub avatar_thumb: Option<String>,
    pub microsoft_uuid: Option<Uuid>,
}

// Add a separate struct for user registration with password
#[derive(Deserialize, Debug)]
pub struct UserRegistration {
    pub name: String,
    pub email: String,
    pub role: String, 
    pub password: String,
    pub pronouns: Option<String>,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub avatar_thumb: Option<String>,
}

// User update struct
#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::users)]
pub struct UserUpdate {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<UserRole>,
    pub pronouns: Option<String>,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub avatar_thumb: Option<String>,
    pub microsoft_uuid: Option<Uuid>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

// User update with password for admin/user management
#[derive(Debug, Serialize, Deserialize)]
pub struct UserUpdateWithPassword {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub pronouns: Option<String>,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub avatar_thumb: Option<String>,
    pub password: Option<String>,
}

// User profile update with ID for profile management
#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfileUpdate {
    pub id: i32,
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub pronouns: Option<String>,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub avatar_thumb: Option<String>,
    pub password: Option<String>,
}

// User response with minimal information
#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i32,
    pub uuid: Uuid,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub pronouns: Option<String>,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub avatar_thumb: Option<String>,
    pub microsoft_uuid: Option<Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

// User info for comments - minimal user data to include with comments
#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub uuid: Uuid,
    pub name: String,
}

// Convert User to UserResponse
impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            uuid: user.uuid,
            name: user.name,
            email: user.email,
            role: user.role,
            pronouns: user.pronouns,
            avatar_url: user.avatar_url,
            banner_url: user.banner_url,
            avatar_thumb: user.avatar_thumb,
            microsoft_uuid: user.microsoft_uuid,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

// Convert User to UserInfo
impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        UserInfo {
            uuid: user.uuid,
            name: user.name,
        }
    }
}

// User Email models for storing multiple email addresses per user
#[derive(Debug, Serialize, Deserialize, Identifiable, Queryable, Associations)]
#[diesel(table_name = crate::schema::user_emails)]
#[diesel(belongs_to(User))]
pub struct UserEmail {
    pub id: i32,
    pub user_id: i32,
    pub email: String,
    pub is_primary: bool,
    pub is_verified: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::user_emails)]
pub struct NewUserEmail {
    pub user_id: i32,
    pub email: String,
    pub is_primary: bool,
    pub is_verified: bool,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::user_emails)]
pub struct UserEmailUpdate {
    pub is_primary: Option<bool>,
    pub is_verified: Option<bool>,
    pub updated_at: Option<NaiveDateTime>,
}

// Extended User response that includes all email addresses
#[derive(Debug, Serialize, Deserialize)]
pub struct UserWithEmails {
    #[serde(flatten)]
    pub user: UserResponse,
    pub emails: Vec<UserEmail>,
}

// Project Status Enum
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)]
#[diesel(sql_type = crate::schema::sql_types::ProjectStatus)]
pub enum ProjectStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "archived")]
    Archived,
}

impl ToSql<crate::schema::sql_types::ProjectStatus, Pg> for ProjectStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let s = match *self {
            ProjectStatus::Active => "active",
            ProjectStatus::Completed => "completed",
            ProjectStatus::Archived => "archived",
        };
        out.write_all(s.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::ProjectStatus, Pg> for ProjectStatus {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"active" => Ok(ProjectStatus::Active),
            b"completed" => Ok(ProjectStatus::Completed),
            b"archived" => Ok(ProjectStatus::Archived),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

// Project model
#[derive(Debug, Serialize, Deserialize, Identifiable, Queryable)]
#[diesel(table_name = crate::schema::projects)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub status: ProjectStatus,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// New Project for creating projects
#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::projects)]
pub struct NewProject {
    pub name: String,
    pub description: Option<String>,
    pub status: ProjectStatus,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

// Project Update for partial updates
#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::projects)]
pub struct ProjectUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<ProjectStatus>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub updated_at: Option<NaiveDateTime>,
}

// Project Ticket association
#[derive(Debug, Serialize, Deserialize, Identifiable, Associations, Queryable)]
#[diesel(belongs_to(Project))]
#[diesel(belongs_to(Ticket))]
#[diesel(table_name = crate::schema::project_tickets)]
#[diesel(primary_key(project_id, ticket_id))]
pub struct ProjectTicket {
    pub project_id: i32,
    pub ticket_id: i32,
}

// New Project Ticket for creating associations
#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::project_tickets)]
pub struct NewProjectTicket {
    pub project_id: i32,
    pub ticket_id: i32,
}

// Project with ticket count for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectWithTicketCount {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub status: ProjectStatus,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub ticket_count: i64,
}

// LinkedTicket model
#[derive(Debug, Serialize, Deserialize, Identifiable, Associations, Queryable)]
#[diesel(table_name = crate::schema::linked_tickets)]
#[diesel(primary_key(ticket_id, linked_ticket_id))]
#[diesel(belongs_to(Ticket, foreign_key = ticket_id))]
pub struct LinkedTicket {
    pub ticket_id: i32,
    pub linked_ticket_id: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::linked_tickets)]
pub struct NewLinkedTicket {
    pub ticket_id: i32,
    pub linked_ticket_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttachmentData {
    pub id: Option<i32>,
    pub url: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewCommentWithAttachments {
    pub content: String,
    pub user_id: i32,
    pub attachments: Vec<AttachmentData>,
}

// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // Subject (user UUID as string for JWT compatibility)
    pub name: String, // User's name
    pub email: String, // User's email
    pub role: String, // User's role
    pub exp: usize,   // Expiration time
    pub iat: usize,   // Issued at
}

// Login request structure
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

// Login response structure
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Deserialize, Debug)]
pub struct PasswordChangeRequest {
    pub current_password: String,
    pub new_password: String,
}

// Add the ArticleContentChunk struct for handling chunked article content
#[derive(Deserialize, Debug)]
pub struct ArticleContentChunk {
    pub chunk_index: i32,
    pub total_chunks: usize,
    pub is_last_chunk: bool,
    pub content: String,
}

// Authentication Provider models
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum AuthProviderType {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "microsoft")]
    Microsoft,
    #[serde(rename = "google")]
    Google,
    #[serde(rename = "saml")]
    Saml,
}

impl ToSql<diesel::sql_types::Text, Pg> for AuthProviderType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let s = match *self {
            AuthProviderType::Local => "local",
            AuthProviderType::Microsoft => "microsoft",
            AuthProviderType::Google => "google",
            AuthProviderType::Saml => "saml",
        };
        out.write_all(s.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<diesel::sql_types::Text, Pg> for AuthProviderType {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"local" => Ok(AuthProviderType::Local),
            b"microsoft" => Ok(AuthProviderType::Microsoft),
            b"google" => Ok(AuthProviderType::Google),
            b"saml" => Ok(AuthProviderType::Saml),
            _ => Err("Unrecognized auth provider type".into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Identifiable, Queryable)]
#[diesel(table_name = crate::schema::auth_providers)]
pub struct AuthProvider {
    pub id: i32,
    pub name: String,
    pub provider_type: String,
    pub enabled: bool,
    pub is_default: bool,
    pub client_id: String,
    pub client_secret: String,
    pub authorization_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scope: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::auth_providers)]
pub struct NewAuthProvider {
    pub name: String,
    pub provider_type: String,
    pub enabled: bool,
    pub is_default: bool,
    pub client_id: String,
    pub client_secret: String,
    pub authorization_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scope: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::auth_providers)]
pub struct AuthProviderUpdate {
    pub name: Option<String>,
    pub provider_type: Option<String>,
    pub enabled: Option<bool>,
    pub is_default: Option<bool>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub authorization_url: Option<String>,
    pub token_url: Option<String>,
    pub redirect_uri: Option<String>,
    pub scope: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}

// Request models for authentication
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthProviderConfigRequest {
    pub provider_id: i32,
    pub configs: Vec<ConfigItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigItem {
    pub key: String,
    pub value: String,
    pub is_secret: bool,
}

// Response model for client display
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthProviderWithConfig {
    pub id: i32,
    pub provider_type: String,
    pub name: String,
    pub enabled: bool,
    pub is_default: bool,
    pub configs: Vec<AuthProviderConfigResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthProviderConfigResponse {
    pub key: String,
    pub value: String,
    pub is_secret: bool,
}

// OAuth state management
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthState {
    pub state: String,
    pub redirect_uri: String,
    pub provider_type: String,
    pub exp: usize,
    pub user_connection: Option<bool>,
}

// OAuth Authentication request
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthRequest {
    pub provider_type: String,
    pub redirect_uri: Option<String>,
    pub user_connection: Option<bool>,
}

// OAuth callback/exchange parameters
#[derive(Debug, Deserialize)]
pub struct OAuthExchangeRequest {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>
}

// Microsoft Entra specific models
#[derive(Debug, Serialize, Deserialize)]
pub struct MicrosoftAuthConfig {
    pub client_id: String,
    pub tenant_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

// Models for user authentication identities
#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Clone)]
#[diesel(table_name = crate::schema::user_auth_identities)]
pub struct UserAuthIdentity {
    pub id: i32,
    pub user_id: i32,
    pub provider_id: i32,
    pub external_id: String,
    pub email: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub password_hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::user_auth_identities)]
pub struct NewUserAuthIdentity {
    pub user_id: i32,
    pub provider_id: i32,
    pub external_id: String,
    pub email: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub password_hash: Option<String>,
}

// For displaying auth identities in the user profile
#[derive(Debug, Serialize, Deserialize)]
pub struct UserAuthIdentityDisplay {
    pub id: i32,
    pub provider_type: String,
    pub provider_name: String,
    pub email: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::documentation_pages)]
pub struct NewDocumentationPage {
    pub uuid: Uuid,
    pub title: String,
    pub slug: Option<String>,
    pub icon: Option<String>,
    pub cover_image: Option<String>,
    pub status: DocumentationStatus,
    pub created_by: Uuid,
    pub last_edited_by: Uuid,
    pub parent_id: Option<i32>,
    pub ticket_id: Option<i32>,
    pub display_order: Option<i32>,
    pub is_public: bool,
    pub is_template: bool,
    pub yjs_state_vector: Option<Vec<u8>>,
    pub yjs_document: Option<Vec<u8>>,
    pub yjs_client_id: Option<i64>,
    pub estimated_reading_time: Option<i32>,
    pub word_count: Option<i32>,
    pub has_unsaved_changes: bool,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::documentation_pages)]
pub struct DocumentationPageUpdate {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub icon: Option<String>,
    pub cover_image: Option<String>,
    pub status: Option<DocumentationStatus>,
    pub last_edited_by: Option<Uuid>,
    pub parent_id: Option<Option<i32>>,
    pub ticket_id: Option<Option<i32>>,
    pub display_order: Option<i32>,
    pub is_public: Option<bool>,
    pub is_template: Option<bool>,
    pub archived_at: Option<Option<chrono::NaiveDateTime>>,
    pub yjs_state_vector: Option<Vec<u8>>,
    pub yjs_document: Option<Vec<u8>>,
    pub yjs_client_id: Option<i64>,
    pub estimated_reading_time: Option<i32>,
    pub word_count: Option<i32>,
    pub has_unsaved_changes: Option<bool>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Clone)]
#[diesel(table_name = crate::schema::documentation_revisions)]
pub struct DocumentationRevision {
    pub id: i32,
    pub page_id: i32,
    pub revision_number: i32,
    pub title: String,
    pub yjs_document_snapshot: Vec<u8>,
    pub yjs_state_vector: Vec<u8>,
    pub created_at: chrono::NaiveDateTime,
    pub created_by: Uuid,
    pub change_summary: Option<String>,
    pub word_count: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::documentation_revisions)]
pub struct NewDocumentationRevision {
    pub page_id: i32,
    pub revision_number: i32,
    pub title: String,
    pub yjs_document_snapshot: Vec<u8>,
    pub yjs_state_vector: Vec<u8>,
    pub created_by: Uuid,
    pub change_summary: Option<String>,
    pub word_count: Option<i32>,
}

// Response models for API
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentationPageResponse {
    pub id: i32,
    pub uuid: Uuid,
    pub title: String,
    pub slug: Option<String>,
    pub icon: Option<String>,
    pub cover_image: Option<String>,
    pub status: DocumentationStatus,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub created_by: UserInfo,
    pub last_edited_by: UserInfo,
    pub parent_id: Option<i32>,
    pub ticket_id: Option<i32>,
    pub display_order: Option<i32>,
    pub is_public: bool,
    pub is_template: bool,
    pub archived_at: Option<chrono::NaiveDateTime>,
    pub estimated_reading_time: Option<i32>,
    pub word_count: Option<i32>,
    pub has_unsaved_changes: bool,
    pub children: Option<Vec<DocumentationPageResponse>>,
}

// Sync History Models
#[derive(Debug, Serialize, Deserialize, Identifiable, Queryable)]
#[diesel(table_name = crate::schema::sync_history)]
pub struct SyncHistory {
    pub id: i32,
    pub sync_type: String,
    pub status: String,
    pub started_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
    pub error_message: Option<String>,
    pub records_processed: Option<i32>,
    pub records_created: Option<i32>,
    pub records_updated: Option<i32>,
    pub records_failed: Option<i32>,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::sync_history)]
pub struct NewSyncHistory {
    pub sync_type: String,
    pub status: String,
    pub started_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
    pub error_message: Option<String>,
    pub records_processed: Option<i32>,
    pub records_created: Option<i32>,
    pub records_updated: Option<i32>,
    pub records_failed: Option<i32>,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::sync_history)]
pub struct SyncHistoryUpdate {
    pub status: Option<String>,
    pub completed_at: Option<Option<NaiveDateTime>>,
    pub error_message: Option<String>,
    pub records_processed: Option<i32>,
    pub records_created: Option<i32>,
    pub records_updated: Option<i32>,
    pub records_failed: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgressPoint {
    pub name: String,
    pub sort_order: i32,
}

// Onboarding models
#[derive(Debug, Serialize, Deserialize)]
pub struct OnboardingStatus {
    pub requires_setup: bool,
    pub user_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct AdminSetupRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminSetupResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<UserResponse>,
}