// models.rs
use chrono::{NaiveDate, NaiveDateTime, DateTime, Utc};
use diesel::prelude::*;
use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::Text;
use serde::{Deserialize, Serialize, Deserializer};
use std::io::Write;

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

#[derive(Debug, Serialize, Deserialize, Identifiable)]
#[diesel(table_name = crate::schema::tickets)]
pub struct Ticket {
    pub id: i32,
    pub title: String,
    pub status: TicketStatus,
    pub priority: TicketPriority,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub assignee: Option<String>,
    pub requester: String,
    pub closed_at: Option<NaiveDateTime>,
}

impl Queryable<(
    diesel::sql_types::Integer,
    diesel::sql_types::Text,
    crate::schema::sql_types::TicketStatus,
    crate::schema::sql_types::TicketPriority,
    diesel::sql_types::Timestamp,
    diesel::sql_types::Timestamp,
    diesel::sql_types::Nullable<diesel::sql_types::Text>,
    diesel::sql_types::Nullable<diesel::sql_types::Text>,
    diesel::sql_types::Nullable<diesel::sql_types::Timestamp>,
), Pg> for Ticket {
    type Row = (
        i32,
        String,
        TicketStatus,
        TicketPriority,
        NaiveDateTime,
        NaiveDateTime,
        Option<String>,
        Option<String>,
        Option<NaiveDateTime>,
    );

    fn build(row: Self::Row) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Ticket {
            id: row.0,
            title: row.1,
            status: row.2,
            priority: row.3,
            created: row.4,
            modified: row.5,
            assignee: row.6,
            requester: row.7.unwrap_or_default(),
            closed_at: row.8,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::tickets)]
pub struct NewTicket {
    pub title: String,
    pub status: TicketStatus,
    pub priority: TicketPriority,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub assignee: Option<String>,
    pub requester: String,
    pub closed_at: Option<NaiveDateTime>,
}

// Add a new struct for partial ticket updates
#[derive(Debug, Serialize, AsChangeset)]
#[diesel(table_name = crate::schema::tickets)]
pub struct TicketUpdate {
    pub title: Option<String>,
    pub status: Option<TicketStatus>,
    pub priority: Option<TicketPriority>,
    pub modified: Option<NaiveDateTime>,
    pub assignee: Option<Option<String>>,
    pub requester: Option<String>,
    pub closed_at: Option<Option<NaiveDateTime>>,
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
            status: Option<TicketStatus>,
            priority: Option<TicketPriority>,
            #[serde(default)]
            modified: Option<String>,
            assignee: Option<Option<String>>,
            requester: Option<String>,
            #[serde(default)]
            closed_at: Option<Option<String>>,
        }

        let helper = TicketUpdateHelper::deserialize(deserializer)?;
        
        // Parse the modified date if it exists
        let modified = if let Some(date_str) = helper.modified {
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

        // Parse the closed_at date if it exists
        let closed_at = if let Some(opt_date_str) = helper.closed_at {
            if let Some(date_str) = opt_date_str {
                // Try to parse the ISO datetime string
                match DateTime::parse_from_rfc3339(&date_str) {
                    Ok(dt) => Some(Some(dt.naive_utc())),
                    Err(_) => {
                        // Fallback to other common formats if RFC3339 fails
                        match NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S") {
                            Ok(dt) => Some(Some(dt)),
                            Err(_) => {
                                // If all parsing attempts fail, use current time
                                println!("Warning: Could not parse closed_at date '{}', using current time", date_str);
                                Some(Some(Utc::now().naive_utc()))
                            }
                        }
                    }
                }
            } else {
                Some(None)
            }
        } else {
            None
        };

        Ok(TicketUpdate {
            title: helper.title,
            status: helper.status,
            priority: helper.priority,
            modified,
            assignee: helper.assignee,
            requester: helper.requester,
            closed_at,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Identifiable, Associations)]
#[diesel(table_name = crate::schema::devices)]
#[diesel(belongs_to(Ticket))]
pub struct Device {
    pub id: i32,
    pub name: String,
    pub hostname: String,
    pub serial_number: String,
    pub model: String,
    pub warranty_status: String,
    pub ticket_id: Option<i32>,
}

impl Queryable<(
    diesel::sql_types::Integer,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Nullable<diesel::sql_types::Integer>,
), Pg> for Device {
    type Row = (
        i32,
        String,
        String,
        String,
        String,
        String,
        Option<i32>,
    );

    fn build(row: Self::Row) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Device {
            id: row.0,
            name: row.1,
            hostname: row.2,
            serial_number: row.3,
            model: row.4,
            warranty_status: row.5,
            ticket_id: row.6,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::devices)]
pub struct NewDevice {
    pub name: String,
    pub hostname: String,
    pub serial_number: String,
    pub model: String,
    pub warranty_status: String,
    pub ticket_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Identifiable, Associations)]
#[diesel(table_name = crate::schema::comments)]
#[diesel(belongs_to(Ticket))]
pub struct Comment {
    pub id: i32,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub user_uuid: String,
    pub ticket_id: i32,
}

impl Queryable<(
    diesel::sql_types::Integer,
    diesel::sql_types::Text,
    diesel::sql_types::Timestamp,
    diesel::sql_types::Text,
    diesel::sql_types::Integer,
), Pg> for Comment {
    type Row = (
        i32,
        String,
        NaiveDateTime,
        String,
        i32,
    );

    fn build(row: Self::Row) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Comment {
            id: row.0,
            content: row.1,
            created_at: row.2,
            user_uuid: row.3,
            ticket_id: row.4,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::comments)]
pub struct NewComment {
    pub content: String,
    pub created_at: Option<NaiveDateTime>,
    pub user_uuid: String,
    pub ticket_id: i32,
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
    pub device: Option<Device>,
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
#[derive(Debug, Serialize, Deserialize, Queryable, Clone)]
#[diesel(table_name = crate::schema::documentation_pages)]
pub struct DocumentationPage {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub content: String,
    pub author: String,
    pub status: DocumentationStatus,
    pub icon: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub parent_id: Option<i32>,
    pub ticket_id: Option<i32>,
    pub display_order: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::documentation_pages)]
pub struct NewDocumentationPage {
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub content: String,
    pub author: String,
    pub status: DocumentationStatus,
    pub icon: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub parent_id: Option<i32>,
    pub ticket_id: Option<i32>,
    pub display_order: Option<i32>,
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
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum UserRole {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "technician")]
    Technician,
    #[serde(rename = "user")]
    User,
}

impl ToSql<diesel::sql_types::Text, Pg> for UserRole {
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

impl FromSql<diesel::sql_types::Text, Pg> for UserRole {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"admin" => Ok(UserRole::Admin),
            b"technician" => Ok(UserRole::Technician),
            b"user" => Ok(UserRole::User),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

// User model
#[derive(Selectable, Identifiable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    #[serde(skip_serializing)]
    pub password_hash: Vec<u8>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Queryable<(
    diesel::sql_types::Integer,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Timestamp,
    diesel::sql_types::Timestamp,
    diesel::sql_types::Binary,
), Pg> for User {
    type Row = (
        i32,
        String,
        String,
        String,
        String,
        NaiveDateTime,
        NaiveDateTime,
        Vec<u8>,
    );

    fn build(row: Self::Row) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let role = match row.4.as_str() {
            "admin" => UserRole::Admin,
            "technician" => UserRole::Technician,
            "user" => UserRole::User,
            _ => UserRole::User, // Default to user if unknown
        };

        Ok(User {
            id: row.0,
            uuid: row.1,
            name: row.2,
            email: row.3,
            role,
            password_hash: row.7,
            created_at: row.5,
            updated_at: row.6,
        })
    }
}

// New user for creation
#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub uuid: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub password_hash: Vec<u8>,
}

// Add a separate struct for user registration with password
#[derive(Deserialize, Debug)]
pub struct UserRegistration {
    pub name: String,
    pub email: String,
    pub role: String, 
    pub password: String,
}

// User update struct
#[derive(Deserialize, Debug)]
pub struct UserUpdateForm {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub password: Option<String>,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = crate::schema::users)]
pub struct UserUpdate {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub password_hash: Option<Vec<u8>>,
}

// User response with minimal information
#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

// User info for comments - minimal user data to include with comments
#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub uuid: String,
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
            role: match user.role {
                UserRole::Admin => "admin".to_string(),
                UserRole::Technician => "technician".to_string(),
                UserRole::User => "user".to_string(),
            },
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
}

// Project Update for partial updates
#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::projects)]
pub struct ProjectUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<ProjectStatus>,
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
    pub user_uuid: String,
    pub attachments: Vec<AttachmentData>,
}

// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // Subject (user UUID)
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
#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleContentChunk {
    pub chunk: String,
    pub chunkIndex: i32,
    pub totalChunks: usize,
    pub isLastChunk: bool,
}

// Add these models for the document_updates table
#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = crate::schema::document_updates)]
pub struct DocumentUpdate {
    pub id: i32,
    pub document_id: String,
    pub update_data: Vec<u8>,
    pub client_id: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crate::schema::document_updates)]
pub struct NewDocumentUpdate {
    pub document_id: String,
    pub client_id: String,
    pub update_data: Vec<u8>,
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
    pub provider_type: String,
    pub name: String,
    pub enabled: bool,
    pub is_default: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::auth_providers)]
pub struct NewAuthProvider {
    pub provider_type: String,
    pub name: String,
    pub enabled: bool,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::auth_providers)]
pub struct AuthProviderUpdate {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub is_default: Option<bool>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Identifiable, Queryable, Associations)]
#[diesel(table_name = crate::schema::auth_provider_configs)]
#[diesel(belongs_to(AuthProvider))]
pub struct AuthProviderConfig {
    pub id: i32,
    pub auth_provider_id: i32,
    pub config_key: String,
    pub config_value: String,
    pub is_secret: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::auth_provider_configs)]
pub struct NewAuthProviderConfig {
    pub auth_provider_id: i32,
    pub config_key: String,
    pub config_value: String,
    pub is_secret: bool,
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
}

// OAuth Authentication request
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthRequest {
    pub provider_type: String,
    pub redirect_uri: Option<String>,
}

// OAuth Exchange Request
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthExchangeRequest {
    pub code: String,
    pub state: String,
}

// Microsoft Entra specific models
#[derive(Debug, Serialize, Deserialize)]
pub struct MicrosoftAuthConfig {
    pub client_id: String,
    pub tenant_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}