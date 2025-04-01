// repository.rs
use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use diesel::result::Error;
use diesel::sql_types::Integer;
use diesel::PgConnection;
use diesel::QueryResult;
use crate::models::*;
use crate::schema::*;
use diesel::sql_types::Text;

use crate::db::DbConnection;
use crate::models::*;
use crate::schema::*;

// Explicitly import UserInfo for clarity
use crate::models::UserInfo;

// Get all tickets
pub fn get_all_tickets(conn: &mut DbConnection) -> QueryResult<Vec<Ticket>> {
    tickets::table.load(conn)
}

// Get paginated tickets with filtering and sorting
pub fn get_paginated_tickets(
    conn: &mut DbConnection,
    page: i64,
    page_size: i64,
    sort_field: Option<String>,
    sort_direction: Option<String>,
    search: Option<String>,
    status: Option<String>,
    priority: Option<String>,
) -> Result<(Vec<Ticket>, i64), Error> {
    // Build the main query
    let mut query = tickets::table.into_boxed();
    
    // Apply filters if provided
    if let Some(search_term) = search.clone() {
        if !search_term.is_empty() {
            let search_pattern = format!("%{}%", search_term.to_lowercase());
            query = query.filter(
                tickets::title.ilike(search_pattern.clone())
                    .or(tickets::id.eq_any(
                        search_term.parse::<i32>().ok().map(|id| vec![id]).unwrap_or_default()
                    ))
                    .or(tickets::requester.ilike(search_pattern.clone()))
                    .or(tickets::assignee.ilike(search_pattern.clone()))
            );
        }
    }
    
    // Handle enum status filter
    if let Some(status_filter) = status.clone() {
        if status_filter != "all" {
            // Convert string to enum
            match status_filter.as_str() {
                "open" => query = query.filter(tickets::status.eq(crate::models::TicketStatus::Open)),
                "in-progress" => query = query.filter(tickets::status.eq(crate::models::TicketStatus::InProgress)),
                "closed" => query = query.filter(tickets::status.eq(crate::models::TicketStatus::Closed)),
                _ => {} // Ignore invalid status values
            }
        }
    }
    
    // Handle enum priority filter
    if let Some(priority_filter) = priority.clone() {
        if priority_filter != "all" {
            // Convert string to enum
            match priority_filter.as_str() {
                "low" => query = query.filter(tickets::priority.eq(crate::models::TicketPriority::Low)),
                "medium" => query = query.filter(tickets::priority.eq(crate::models::TicketPriority::Medium)),
                "high" => query = query.filter(tickets::priority.eq(crate::models::TicketPriority::High)),
                _ => {} // Ignore invalid priority values
            }
        }
    }
    
    // Build a separate count query with the same filters
    let mut count_query = tickets::table.into_boxed();
    
    // Apply the same filters to the count query
    if let Some(search_term) = search {
        if !search_term.is_empty() {
            let search_pattern = format!("%{}%", search_term.to_lowercase());
            count_query = count_query.filter(
                tickets::title.ilike(search_pattern.clone())
                    .or(tickets::id.eq_any(
                        search_term.parse::<i32>().ok().map(|id| vec![id]).unwrap_or_default()
                    ))
                    .or(tickets::requester.ilike(search_pattern.clone()))
                    .or(tickets::assignee.ilike(search_pattern))
            );
        }
    }
    
    // Handle enum status filter for count query
    if let Some(status_filter) = status {
        if status_filter != "all" {
            // Convert string to enum
            match status_filter.as_str() {
                "open" => count_query = count_query.filter(tickets::status.eq(crate::models::TicketStatus::Open)),
                "in-progress" => count_query = count_query.filter(tickets::status.eq(crate::models::TicketStatus::InProgress)),
                "closed" => count_query = count_query.filter(tickets::status.eq(crate::models::TicketStatus::Closed)),
                _ => {} // Ignore invalid status values
            }
        }
    }
    
    // Handle enum priority filter for count query
    if let Some(priority_filter) = priority {
        if priority_filter != "all" {
            // Convert string to enum
            match priority_filter.as_str() {
                "low" => count_query = count_query.filter(tickets::priority.eq(crate::models::TicketPriority::Low)),
                "medium" => count_query = count_query.filter(tickets::priority.eq(crate::models::TicketPriority::Medium)),
                "high" => count_query = count_query.filter(tickets::priority.eq(crate::models::TicketPriority::High)),
                _ => {} // Ignore invalid priority values
            }
        }
    }
    
    // Count total matching records (before pagination)
    let total: i64 = count_query.count().get_result(conn)?;
    
    // Apply sorting to the main query
    match (sort_field.as_deref(), sort_direction.as_deref()) {
        (Some("id"), Some("asc")) => query = query.order(tickets::id.asc()),
        (Some("id"), _) => query = query.order(tickets::id.desc()),
        (Some("title"), Some("asc")) => query = query.order(tickets::title.asc()),
        (Some("title"), _) => query = query.order(tickets::title.desc()),
        (Some("status"), Some("asc")) => query = query.order(tickets::status.asc()),
        (Some("status"), _) => query = query.order(tickets::status.desc()),
        (Some("priority"), Some("asc")) => query = query.order(tickets::priority.asc()),
        (Some("priority"), _) => query = query.order(tickets::priority.desc()),
        (Some("created"), Some("asc")) => query = query.order(tickets::created.asc()),
        (Some("created"), _) => query = query.order(tickets::created.desc()),
        (Some("requester"), Some("asc")) => query = query.order(tickets::requester.asc()),
        (Some("requester"), _) => query = query.order(tickets::requester.desc()),
        (Some("assignee"), Some("asc")) => query = query.order(tickets::assignee.asc()),
        (Some("assignee"), _) => query = query.order(tickets::assignee.desc()),
        _ => query = query.order(tickets::id.desc()), // Default sort
    }
    
    // Apply pagination
    let offset = (page - 1) * page_size;
    query = query.offset(offset).limit(page_size);
    
    // Execute the query
    let results = query.load::<Ticket>(conn)?;
    
    Ok((results, total))
}

pub fn get_ticket_by_id(conn: &mut DbConnection, ticket_id: i32) -> QueryResult<Ticket> {
    tickets::table
        .find(ticket_id)
        .first(conn)
}

pub fn create_ticket(conn: &mut DbConnection, new_ticket: NewTicket) -> QueryResult<Ticket> {
    diesel::insert_into(tickets::table)
        .values(&new_ticket)
        .get_result(conn)
}

pub fn update_ticket(conn: &mut DbConnection, ticket_id: i32, ticket: NewTicket) -> QueryResult<Ticket> {
    diesel::update(tickets::table.find(ticket_id))
        .set(&ticket)
        .get_result(conn)
}

// Add a new function for partial ticket updates
pub fn update_ticket_partial(conn: &mut DbConnection, ticket_id: i32, ticket_update: crate::models::TicketUpdate) -> QueryResult<Ticket> {
    println!("Updating ticket {} with: {:?}", ticket_id, ticket_update);
    
    diesel::update(tickets::table.find(ticket_id))
        .set(&ticket_update)
        .get_result(conn)
}

pub fn delete_ticket(conn: &mut DbConnection, ticket_id: i32) -> QueryResult<usize> {
    diesel::delete(tickets::table.find(ticket_id)).execute(conn)
}

// Device operations
pub fn get_all_devices(conn: &mut DbConnection) -> QueryResult<Vec<Device>> {
    devices::table
        .order_by(devices::id.asc())
        .load::<Device>(conn)
}

pub fn get_device_by_id(conn: &mut DbConnection, device_id: i32) -> QueryResult<Device> {
    devices::table
        .find(device_id)
        .first(conn)
}

pub fn get_device_by_ticket_id(conn: &mut DbConnection, ticket_id: i32) -> QueryResult<Device> {
    devices::table
        .filter(devices::ticket_id.eq(ticket_id))
        .first(conn)
}

pub fn create_device(conn: &mut DbConnection, new_device: NewDevice) -> QueryResult<Device> {
    diesel::insert_into(devices::table)
        .values(&new_device)
        .get_result(conn)
}

pub fn update_device(conn: &mut DbConnection, device_id: i32, device: NewDevice) -> QueryResult<Device> {
    diesel::update(devices::table.find(device_id))
        .set(&device)
        .get_result(conn)
}

pub fn delete_device(conn: &mut DbConnection, device_id: i32) -> QueryResult<usize> {
    diesel::delete(devices::table.find(device_id))
        .execute(conn)
}

// Comment operations
pub fn get_comments_by_ticket_id(conn: &mut DbConnection, ticket_id: i32) -> QueryResult<Vec<Comment>> {
    comments::table
        .filter(comments::ticket_id.eq(ticket_id))
        .order(comments::created_at.desc())
        .load(conn)
}

pub fn create_comment(conn: &mut DbConnection, new_comment: NewComment) -> QueryResult<Comment> {
    diesel::insert_into(comments::table)
        .values(&new_comment)
        .get_result(conn)
}

// Attachment operations
pub fn get_attachments_by_comment_id(conn: &mut DbConnection, comment_id: i32) -> QueryResult<Vec<Attachment>> {
    attachments::table
        .filter(attachments::comment_id.eq(comment_id))
        .load(conn)
}

pub fn create_attachment(conn: &mut DbConnection, new_attachment: NewAttachment) -> QueryResult<Attachment> {
    diesel::insert_into(attachments::table)
        .values(&new_attachment)
        .get_result(conn)
}

// Article content operations
pub fn get_article_content_by_ticket_id(conn: &mut DbConnection, ticket_id: i32) -> QueryResult<ArticleContent> {
    article_contents::table
        .filter(article_contents::ticket_id.eq(ticket_id))
        .first(conn)
}

pub fn create_article_content(conn: &mut DbConnection, new_article_content: NewArticleContent) -> QueryResult<ArticleContent> {
    diesel::insert_into(article_contents::table)
        .values(&new_article_content)
        .get_result(conn)
}

pub fn update_article_content(conn: &mut DbConnection, ticket_id: i32, article_content: NewArticleContent) -> QueryResult<ArticleContent> {
    // First check if article content exists for this ticket
    let existing = article_contents::table
        .filter(article_contents::ticket_id.eq(ticket_id))
        .first::<ArticleContent>(conn);
    
    match existing {
        Ok(article) => {
            // Update existing article content
            diesel::update(article_contents::table.find(article.id))
                .set(&article_content)
                .get_result(conn)
        },
        Err(Error::NotFound) => {
            // Create new article content if it doesn't exist
            create_article_content(conn, article_content)
        },
        Err(e) => Err(e)
    }
}

// Composite operations
pub fn get_complete_ticket(conn: &mut DbConnection, ticket_id: i32) -> Result<CompleteTicket, Error> {
    println!("Getting complete ticket data for ticket ID: {}", ticket_id);
    
    let ticket = get_ticket_by_id(conn, ticket_id)?;
    println!("Found ticket: {} - {}", ticket.id, ticket.title);
    
    let device = get_device_by_ticket_id(conn, ticket_id).ok();
    
    let comments = get_comments_by_ticket_id(conn, ticket_id)?;
    let mut comments_with_attachments = Vec::new();
    
    for comment in comments {
        let attachments = get_attachments_by_comment_id(conn, comment.id)?;
        
        // Get user information for this comment
        let user = match get_user_by_uuid(&comment.user_uuid, conn) {
            Ok(user) => Some(UserInfo::from(user)),
            Err(_) => None,
        };
        
        comments_with_attachments.push(CommentWithAttachments {
            comment,
            attachments,
            user,
        });
    }
    
    let article_content = get_article_content_by_ticket_id(conn, ticket_id)
        .map(|content| content.content)
        .ok();
    
    // Get linked tickets
    let linked_tickets = get_linked_tickets(conn, ticket_id).unwrap_or_default();
    println!("Linked tickets for ticket {}: {:?}", ticket_id, linked_tickets);
    
    // Get projects for this ticket
    let projects = get_projects_for_ticket(conn, ticket_id).unwrap_or_default();
    println!("Projects for ticket {}: {:?}", ticket_id, projects.len());
    
    Ok(CompleteTicket {
        ticket,
        device,
        comments: comments_with_attachments,
        article_content,
        linked_tickets,
        projects,
    })
}

// Import from JSON
pub fn import_ticket_from_json(conn: &mut DbConnection, ticket_json: &TicketJson) -> Result<Ticket, Error> {
    // Parse status
    let status = match ticket_json.status.as_str() {
        "open" => TicketStatus::Open,
        "in-progress" => TicketStatus::InProgress,
        "closed" => TicketStatus::Closed,
        _ => TicketStatus::Open, // Default to open if unknown
    };
    
    // Parse priority
    let priority = match ticket_json.priority.as_str() {
        "low" => TicketPriority::Low,
        "medium" => TicketPriority::Medium,
        "high" => TicketPriority::High,
        _ => TicketPriority::Medium, // Default to medium if unknown
    };
    
    // Parse dates
    // Parse created as NaiveDateTime
    let created = NaiveDateTime::parse_from_str(&ticket_json.created, "%Y-%m-%dT%H:%M:%S")
        .unwrap_or_else(|_| chrono::Local::now().naive_local());
    
    // Parse modified as NaiveDateTime
    let modified = NaiveDateTime::parse_from_str(&format!("{} 00:00:00", ticket_json.modified), "%Y-%m-%d %H:%M:%S")
        .unwrap_or_else(|_| chrono::Local::now().naive_local());
    
    // Create the ticket
    let new_ticket = NewTicket {
        title: ticket_json.title.clone(),
        status,
        priority,
        created,
        modified,
        assignee: if ticket_json.assignee.is_empty() { None } else { Some(ticket_json.assignee.clone()) },
        requester: ticket_json.requester.clone(),
        closed_at: None,
    };
    
    let ticket = create_ticket(conn, new_ticket)?;
    
    // Create device if present
    if let Some(device_json) = &ticket_json.device {
        let new_device = NewDevice {
            name: device_json.name.clone(),
            hostname: device_json.hostname.clone(),
            serial_number: device_json.serial_number.clone(),
            model: device_json.model.clone(),
            warranty_status: device_json.warranty_status.clone(),
            ticket_id: ticket.id,
        };
        
        create_device(conn, new_device)?;
    }
    
    // Create comments and attachments if present
    if let Some(comments_json) = &ticket_json.comments {
        for comment_json in comments_json {
            let created_at = NaiveDateTime::parse_from_str(&comment_json.created_at, "%Y-%m-%dT%H:%M:%S")
                .unwrap_or_else(|_| chrono::Local::now().naive_local());
            
            let new_comment = NewComment {
                content: comment_json.content.clone(),
                user_uuid: comment_json.user_uuid.clone(),
                created_at: Some(created_at),
                ticket_id: ticket.id,
            };
            
            let comment = create_comment(conn, new_comment)?;
            
            // Create attachments for this comment
            for attachment_json in &comment_json.attachments {
                let new_attachment = NewAttachment {
                    url: attachment_json.url.clone(),
                    name: attachment_json.name.clone(),
                    comment_id: Some(comment.id),
                };
                
                create_attachment(conn, new_attachment)?;
            }
        }
    }
    
    // Create article content if present
    if let Some(content) = &ticket_json.article_content {
        let new_article_content = NewArticleContent {
            content: content.clone(),
            ticket_id: ticket.id,
        };
        
        create_article_content(conn, new_article_content)?;
    }
    
    Ok(ticket)
}

// Documentation Repository Functions
use crate::models::{
    DocumentationPage, DocumentationPageWithChildren,
    NewDocumentationPage, DocumentationStatus,
};
use crate::schema::{documentation_pages};

// Documentation Page Functions
pub fn get_documentation_pages(conn: &mut PgConnection) -> Result<Vec<DocumentationPage>, diesel::result::Error> {
    documentation_pages::table
        .order_by(documentation_pages::title.asc())
        .load::<DocumentationPage>(conn)
}

pub fn get_documentation_page(id: i32, conn: &mut PgConnection) -> Result<DocumentationPage, diesel::result::Error> {
    documentation_pages::table
        .find(id)
        .first::<DocumentationPage>(conn)
}

pub fn get_documentation_page_by_slug(slug: &str, conn: &mut PgConnection) -> Result<DocumentationPage, diesel::result::Error> {
    documentation_pages::table
        .filter(documentation_pages::slug.eq(slug))
        .first::<DocumentationPage>(conn)
}

pub fn create_documentation_page(
    page: NewDocumentationPage,
    conn: &mut PgConnection,
) -> Result<DocumentationPage, diesel::result::Error> {
    diesel::insert_into(documentation_pages::table)
        .values(page)
        .get_result(conn)
}

pub fn update_documentation_page(
    conn: &mut PgConnection,
    page: &DocumentationPage,
) -> Result<DocumentationPage, diesel::result::Error> {
    use crate::schema::documentation_pages;

    diesel::update(documentation_pages::table.find(page.id))
        .set((
            documentation_pages::slug.eq(page.slug.clone()),
            documentation_pages::title.eq(page.title.clone()),
            documentation_pages::description.eq(page.description.clone()),
            documentation_pages::content.eq(page.content.clone()),
            documentation_pages::parent_id.eq(page.parent_id),
            documentation_pages::author.eq(page.author.clone()),
            documentation_pages::status.eq(page.status),
            documentation_pages::icon.eq(page.icon.clone()),
            documentation_pages::updated_at.eq(page.updated_at),
        ))
        .get_result(conn)
}

pub fn delete_documentation_page(id: i32, conn: &mut PgConnection) -> Result<usize, diesel::result::Error> {
    diesel::delete(documentation_pages::table.find(id)).execute(conn)
}

// New functions for wiki-like documentation structure
pub fn get_top_level_pages(conn: &mut PgConnection) -> Result<Vec<DocumentationPage>, diesel::result::Error> {
    documentation_pages::table
        .filter(documentation_pages::parent_id.is_null())
        .order_by(documentation_pages::title.asc())
        .load::<DocumentationPage>(conn)
}

pub fn get_pages_by_parent_id(parent_id: i32, conn: &mut PgConnection) -> Result<Vec<DocumentationPage>, diesel::result::Error> {
    documentation_pages::table
        .filter(documentation_pages::parent_id.eq(parent_id))
        .order_by(documentation_pages::title.asc())
        .load::<DocumentationPage>(conn)
}

// Get page with children using parent_id
pub fn get_page_with_children_by_parent_id(
    id: i32,
    conn: &mut PgConnection,
) -> Result<DocumentationPageWithChildren, diesel::result::Error> {
    let page = get_documentation_page(id, conn)?;
    let children = get_pages_by_parent_id(id, conn)?;
    
    Ok(DocumentationPageWithChildren {
        page,
        children,
    })
}

// User repository functions
pub fn get_users(conn: &mut PgConnection) -> Result<Vec<User>, diesel::result::Error> {
    users::table
        .order_by(users::name.asc())
        .load::<User>(conn)
}

pub fn get_user_by_id(id: i32, conn: &mut PgConnection) -> Result<User, diesel::result::Error> {
    users::table
        .find(id)
        .first::<User>(conn)
}

pub fn get_user_by_uuid(uuid: &str, conn: &mut PgConnection) -> Result<User, diesel::result::Error> {
    users::table
        .filter(users::uuid.eq(uuid))
        .first::<User>(conn)
}

pub fn get_user_by_email(email: &str, conn: &mut PgConnection) -> Result<User, diesel::result::Error> {
    users::table
        .filter(users::email.eq(email))
        .first::<User>(conn)
}

pub fn create_user(
    user: NewUser,
    conn: &mut PgConnection,
) -> Result<User, diesel::result::Error> {
    diesel::insert_into(users::table)
        .values(user)
        .get_result(conn)
}

pub fn update_user(
    id: i32,
    user: UserUpdate,
    conn: &mut PgConnection,
) -> Result<User, diesel::result::Error> {
    diesel::update(users::table.find(id))
        .set(user)
        .get_result(conn)
}

pub fn delete_user(id: i32, conn: &mut PgConnection) -> Result<usize, diesel::result::Error> {
    diesel::delete(users::table.find(id)).execute(conn)
}

// Project operations
pub fn get_all_projects(conn: &mut DbConnection) -> QueryResult<Vec<Project>> {
    projects::table.load(conn)
}

pub fn get_projects_with_ticket_count(conn: &mut DbConnection) -> Result<Vec<ProjectWithTicketCount>, Error> {
    // Get all projects
    let all_projects = projects::table.load::<Project>(conn)?;
    
    // For each project, count the tickets
    let mut projects_with_count = Vec::new();
    
    for project in all_projects {
        let count = project_tickets::table
            .filter(project_tickets::project_id.eq(project.id))
            .count()
            .get_result::<i64>(conn)?;
        
        projects_with_count.push(ProjectWithTicketCount {
            id: project.id,
            name: project.name,
            description: project.description,
            status: project.status,
            created_at: project.created_at,
            updated_at: project.updated_at,
            ticket_count: count,
        });
    }
    
    Ok(projects_with_count)
}

pub fn get_project_by_id(conn: &mut DbConnection, project_id: i32) -> QueryResult<Project> {
    projects::table
        .find(project_id)
        .first(conn)
}

pub fn get_project_with_ticket_count(conn: &mut DbConnection, project_id: i32) -> Result<ProjectWithTicketCount, Error> {
    let project = projects::table
        .find(project_id)
        .first::<Project>(conn)?;
    
    let count = project_tickets::table
        .filter(project_tickets::project_id.eq(project_id))
        .count()
        .get_result::<i64>(conn)?;
    
    Ok(ProjectWithTicketCount {
        id: project.id,
        name: project.name,
        description: project.description,
        status: project.status,
        created_at: project.created_at,
        updated_at: project.updated_at,
        ticket_count: count,
    })
}

pub fn create_project(conn: &mut DbConnection, new_project: NewProject) -> QueryResult<Project> {
    diesel::insert_into(projects::table)
        .values(&new_project)
        .get_result(conn)
}

pub fn update_project(conn: &mut DbConnection, project_id: i32, project_update: ProjectUpdate) -> QueryResult<Project> {
    // Set updated_at to current time if not provided
    let project_update = if project_update.updated_at.is_none() {
        let mut update = project_update;
        update.updated_at = Some(chrono::Utc::now().naive_utc());
        update
    } else {
        project_update
    };
    
    diesel::update(projects::table.find(project_id))
        .set(&project_update)
        .get_result(conn)
}

pub fn delete_project(conn: &mut DbConnection, project_id: i32) -> QueryResult<usize> {
    // This will also delete all project_tickets entries due to ON DELETE CASCADE
    diesel::delete(projects::table.find(project_id)).execute(conn)
}

// Project-Ticket association operations
pub fn add_ticket_to_project(conn: &mut DbConnection, project_id: i32, ticket_id: i32) -> QueryResult<ProjectTicket> {
    // First check if the ticket exists
    match get_ticket_by_id(conn, ticket_id) {
        Ok(_) => println!("Ticket {} exists", ticket_id),
        Err(e) => {
            println!("Error: Ticket {} does not exist: {:?}", ticket_id, e);
            return Err(Error::NotFound);
        }
    }

    // Then check if the project exists
    match projects::table.find(project_id).first::<Project>(conn) {
        Ok(_) => println!("Project {} exists", project_id),
        Err(e) => {
            println!("Error: Project {} does not exist: {:?}", project_id, e);
            return Err(Error::NotFound);
        }
    }
    
    // Check if the association already exists
    let existing = project_tickets::table
        .filter(project_tickets::project_id.eq(project_id))
        .filter(project_tickets::ticket_id.eq(ticket_id))
        .first::<ProjectTicket>(conn);
    
    if let Ok(association) = existing {
        println!("Association already exists between project {} and ticket {}", project_id, ticket_id);
        return Ok(association);
    }
    
    let new_association = NewProjectTicket {
        project_id,
        ticket_id,
    };
    
    println!("Creating new association between project {} and ticket {}", project_id, ticket_id);
    diesel::insert_into(project_tickets::table)
        .values(&new_association)
        .get_result(conn)
}

pub fn remove_ticket_from_project(conn: &mut DbConnection, project_id: i32, ticket_id: i32) -> QueryResult<usize> {
    diesel::delete(
        project_tickets::table
            .filter(project_tickets::project_id.eq(project_id))
            .filter(project_tickets::ticket_id.eq(ticket_id))
    ).execute(conn)
}

pub fn get_project_tickets(conn: &mut DbConnection, project_id: i32) -> QueryResult<Vec<Ticket>> {
    project_tickets::table
        .filter(project_tickets::project_id.eq(project_id))
        .inner_join(tickets::table)
        .select(tickets::all_columns)
        .load::<Ticket>(conn)
}

// Linked Tickets
pub fn get_linked_tickets(conn: &mut DbConnection, ticket_id: i32) -> QueryResult<Vec<i32>> {
    use crate::schema::linked_tickets;
    use diesel::prelude::*;
    
    println!("Getting linked tickets for ticket ID: {}", ticket_id);
    
    // Use explicit table and column references to avoid ambiguity
    let linked_ids = linked_tickets::table
        .filter(linked_tickets::ticket_id.eq(ticket_id))
        .select(linked_tickets::linked_ticket_id)
        .load::<i32>(conn)?;
        
    println!("Found {} linked tickets for ticket ID {}: {:?}", linked_ids.len(), ticket_id, linked_ids);
    
    Ok(linked_ids)
}

pub fn link_tickets(conn: &mut DbConnection, ticket1_id: i32, ticket2_id: i32) -> QueryResult<()> {
    use crate::schema::linked_tickets;
    
    // Print debug information
    println!("Linking tickets: {} and {}", ticket1_id, ticket2_id);
    
    // First, check if the tickets exist
    let ticket1 = get_ticket_by_id(conn, ticket1_id)?;
    let ticket2 = get_ticket_by_id(conn, ticket2_id)?;
    
    println!("Found ticket1: {} - {}", ticket1.id, ticket1.title);
    println!("Found ticket2: {} - {}", ticket2.id, ticket2.title);
    
    // Check if the links already exist
    let existing_links_1_to_2 = linked_tickets::table
        .filter(linked_tickets::ticket_id.eq(ticket1_id))
        .filter(linked_tickets::linked_ticket_id.eq(ticket2_id))
        .count()
        .get_result::<i64>(conn)?;
        
    let existing_links_2_to_1 = linked_tickets::table
        .filter(linked_tickets::ticket_id.eq(ticket2_id))
        .filter(linked_tickets::linked_ticket_id.eq(ticket1_id))
        .count()
        .get_result::<i64>(conn)?;
        
    println!("Found {} existing links from ticket {} to {}", existing_links_1_to_2, ticket1_id, ticket2_id);
    println!("Found {} existing links from ticket {} to {}", existing_links_2_to_1, ticket2_id, ticket1_id);
    
    // Create bidirectional links
    let new_link1 = NewLinkedTicket {
        ticket_id: ticket1.id,
        linked_ticket_id: ticket2.id,
    };
    
    let new_link2 = NewLinkedTicket {
        ticket_id: ticket2.id,
        linked_ticket_id: ticket1.id,
    };
    
    // Insert both links in a transaction
    conn.transaction(|conn| {
        let inserted_1_to_2 = diesel::insert_into(linked_tickets::table)
            .values(&new_link1)
            .on_conflict_do_nothing()
            .execute(conn)?;
            
        let inserted_2_to_1 = diesel::insert_into(linked_tickets::table)
            .values(&new_link2)
            .on_conflict_do_nothing()
            .execute(conn)?;
            
        println!("Inserted {} links from ticket {} to {}", inserted_1_to_2, ticket1_id, ticket2_id);
        println!("Inserted {} links from ticket {} to {}", inserted_2_to_1, ticket2_id, ticket1_id);
        
        Ok(())
    })
}

pub fn unlink_tickets(conn: &mut DbConnection, ticket1_id: i32, ticket2_id: i32) -> QueryResult<()> {
    use crate::schema::linked_tickets::dsl::*;
    
    // Print debug information
    println!("Unlinking tickets: {} and {}", ticket1_id, ticket2_id);
    
    // Check if the links exist before attempting to delete
    let links_from_1_to_2 = linked_tickets
        .filter(ticket_id.eq(ticket1_id))
        .filter(linked_ticket_id.eq(ticket2_id))
        .count()
        .get_result::<i64>(conn)?;
        
    let links_from_2_to_1 = linked_tickets
        .filter(ticket_id.eq(ticket2_id))
        .filter(linked_ticket_id.eq(ticket1_id))
        .count()
        .get_result::<i64>(conn)?;
        
    println!("Found {} links from ticket {} to {}", links_from_1_to_2, ticket1_id, ticket2_id);
    println!("Found {} links from ticket {} to {}", links_from_2_to_1, ticket2_id, ticket1_id);
    
    // Delete both links in a transaction
    conn.transaction(|conn| {
        // Delete link from ticket1 to ticket2
        let deleted_1_to_2 = diesel::delete(
            linked_tickets
                .filter(ticket_id.eq(ticket1_id))
                .filter(linked_ticket_id.eq(ticket2_id))
        ).execute(conn)?;
        
        // Delete link from ticket2 to ticket1
        let deleted_2_to_1 = diesel::delete(
            linked_tickets
                .filter(ticket_id.eq(ticket2_id))
                .filter(linked_ticket_id.eq(ticket1_id))
        ).execute(conn)?;
        
        println!("Deleted {} links from ticket {} to {}", deleted_1_to_2, ticket1_id, ticket2_id);
        println!("Deleted {} links from ticket {} to {}", deleted_2_to_1, ticket2_id, ticket1_id);
        
        Ok(())
    })
}

// Get projects for a ticket
pub fn get_projects_for_ticket(conn: &mut DbConnection, ticket_id: i32) -> QueryResult<Vec<Project>> {
    println!("Getting projects for ticket ID: {}", ticket_id);
    
    project_tickets::table
        .filter(project_tickets::ticket_id.eq(ticket_id))
        .inner_join(projects::table)
        .select(projects::all_columns)
        .load::<Project>(conn)
}

// Get a comment by ID
pub fn get_comment_by_id(conn: &mut DbConnection, comment_id: i32) -> QueryResult<Comment> {
    comments::table.find(comment_id).first(conn)
}

// Get comments with attachments by ticket ID
pub fn get_comments_with_attachments_by_ticket_id(conn: &mut DbConnection, ticket_id: i32) -> QueryResult<Vec<CommentWithAttachments>> {
    let comments = get_comments_by_ticket_id(conn, ticket_id)?;
    let mut comments_with_attachments = Vec::new();
    
    for comment in comments {
        let attachments = get_attachments_by_comment_id(conn, comment.id)?;
        
        // Get user information for this comment
        let user = match get_user_by_uuid(&comment.user_uuid, conn) {
            Ok(user) => Some(UserInfo::from(user)),
            Err(_) => None,
        };
        
        comments_with_attachments.push(CommentWithAttachments {
            comment,
            attachments,
            user,
        });
    }
    
    Ok(comments_with_attachments)
}

// Delete a comment and its attachments
pub fn delete_comment(conn: &mut DbConnection, comment_id: i32) -> QueryResult<usize> {
    // First delete all attachments associated with this comment
    diesel::delete(attachments::table.filter(attachments::comment_id.eq(comment_id))).execute(conn)?;
    
    // Then delete the comment itself
    diesel::delete(comments::table.find(comment_id)).execute(conn)
}

pub fn get_documentation_pages_by_ticket_id(conn: &mut PgConnection, ticket_id: i32) -> Result<Vec<DocumentationPage>, diesel::result::Error> {
    use crate::schema::documentation_pages;
    
    documentation_pages::table
        .filter(documentation_pages::ticket_id.eq(ticket_id))
        .order_by(documentation_pages::title.asc())
        .load::<DocumentationPage>(conn)
}

pub fn get_documentation_pages_by_parent_id(
    conn: &mut PgConnection,
    parent_id: i32,
) -> Result<Vec<DocumentationPage>, diesel::result::Error> {
    use crate::schema::documentation_pages;

    documentation_pages::table
        .filter(documentation_pages::parent_id.eq(parent_id))
        .load::<DocumentationPage>(conn)
}

// New repository functions for page relationships and ordering

// Get top-level documentation pages with correct ordering
pub fn get_ordered_top_level_pages(
    conn: &mut PgConnection,
) -> Result<Vec<DocumentationPage>, diesel::result::Error> {
    use crate::schema::{documentation_pages};
    use diesel::sql_types::{Integer, Nullable};

    // Define a SQL function for coalesce
    sql_function! {
        fn coalesce(x: Nullable<Integer>, y: Integer) -> Integer;
    }

    // Get top-level pages ordered by display_order, treating NULL as 0
    documentation_pages::table
        .filter(documentation_pages::parent_id.is_null())
        .order_by(coalesce(documentation_pages::display_order, 0).asc())
        .load::<DocumentationPage>(conn)
}

// Get documentation pages by parent ID with correct ordering
pub fn get_ordered_pages_by_parent_id(
    conn: &mut PgConnection,
    parent_id: i32,
) -> Result<Vec<DocumentationPage>, diesel::result::Error> {
    use crate::schema::{documentation_pages};
    use diesel::sql_types::{Integer, Nullable};

    // Define a SQL function for coalesce
    sql_function! {
        fn coalesce(x: Nullable<Integer>, y: Integer) -> Integer;
    }

    // Get pages with the specified parent_id, ordered by display_order, treating NULL as 0
    documentation_pages::table
        .filter(documentation_pages::parent_id.eq(parent_id))
        .order_by(coalesce(documentation_pages::display_order, 0).asc())
        .load::<DocumentationPage>(conn)
}

// Reorder pages under a parent
pub fn reorder_pages(
    conn: &mut PgConnection,
    parent_id: Option<i32>,
    page_orders: &[PageOrder],
) -> Result<Vec<DocumentationPage>, diesel::result::Error> {
    use crate::schema::{documentation_pages};
    
    // Begin transaction
    conn.transaction(|conn| {
        let mut updated_pages = Vec::new();
        
        for order in page_orders {
            // Update the page's display_order and ensure it has the correct parent_id
            let updated_page = diesel::update(documentation_pages::table.find(order.page_id))
                .set((
                    documentation_pages::display_order.eq(order.display_order),
                    documentation_pages::parent_id.eq(parent_id),
                ))
                .get_result::<DocumentationPage>(conn)?;
                
            updated_pages.push(updated_page);
        }
        
        Ok(updated_pages)
    })
}

// Move a page to a new parent
pub fn move_page_to_parent(
    conn: &mut PgConnection,
    page_id: i32,
    new_parent_id: Option<i32>,
    display_order: i32,
) -> Result<DocumentationPage, diesel::result::Error> {
    use crate::schema::{documentation_pages};
    
    // Begin transaction
    conn.transaction(|conn| {
        // Update the page's parent_id and display_order
        let updated_page = diesel::update(documentation_pages::table.find(page_id))
            .set((
                documentation_pages::parent_id.eq(new_parent_id),
                documentation_pages::display_order.eq(display_order),
            ))
            .get_result::<DocumentationPage>(conn)?;
            
        Ok(updated_page)
    })
}

// Get page with ordered children
pub fn get_page_with_ordered_children(
    conn: &mut PgConnection,
    page_id: i32,
) -> Result<DocumentationPageWithChildren, diesel::result::Error> {
    let page = get_documentation_page(page_id, conn)?;
    let children = get_ordered_pages_by_parent_id(conn, page_id)?;
    
    Ok(DocumentationPageWithChildren {
        page,
        children,
    })
}

// Get an attachment by ID
pub fn get_attachment_by_id(conn: &mut DbConnection, attachment_id: i32) -> QueryResult<Attachment> {
    attachments::table
        .find(attachment_id)
        .first(conn)
}

// Delete an attachment
pub fn delete_attachment(conn: &mut DbConnection, attachment_id: i32) -> QueryResult<usize> {
    diesel::delete(attachments::table.find(attachment_id))
        .execute(conn)
}

pub fn create_complete_ticket(conn: &mut DbConnection, ticket_json: TicketJson) -> Result<Ticket, diesel::result::Error> {
    // Parse status
    let status = match ticket_json.status.as_str() {
        "open" => TicketStatus::Open,
        "in-progress" => TicketStatus::InProgress,
        "closed" => TicketStatus::Closed,
        _ => TicketStatus::Open, // Default to open if unknown
    };
    
    // Parse priority
    let priority = match ticket_json.priority.as_str() {
        "low" => TicketPriority::Low,
        "medium" => TicketPriority::Medium,
        "high" => TicketPriority::High,
        _ => TicketPriority::Medium, // Default to medium if unknown
    };
    
    // Parse dates
    // Parse created as NaiveDateTime
    let created = NaiveDateTime::parse_from_str(&ticket_json.created, "%Y-%m-%dT%H:%M:%S")
        .unwrap_or_else(|_| chrono::Local::now().naive_local());
    
    // Parse modified as NaiveDateTime
    let modified = NaiveDateTime::parse_from_str(&ticket_json.modified, "%Y-%m-%dT%H:%M:%S")
        .unwrap_or_else(|_| chrono::Local::now().naive_local());
    
    // Create the ticket
    let new_ticket = NewTicket {
        title: ticket_json.title.clone(),
        status,
        priority,
        created,
        modified,
        assignee: if ticket_json.assignee.is_empty() { None } else { Some(ticket_json.assignee.clone()) },
        requester: ticket_json.requester.clone(),
        closed_at: None,
    };
    
    let ticket = create_ticket(conn, new_ticket)?;
    
    // Create device if present
    if let Some(device_json) = &ticket_json.device {
        let new_device = NewDevice {
            name: device_json.name.clone(),
            hostname: device_json.hostname.clone(),
            serial_number: device_json.serial_number.clone(),
            model: device_json.model.clone(),
            warranty_status: device_json.warranty_status.clone(),
            ticket_id: ticket.id,
        };
        
        create_device(conn, new_device)?;
    }
    
    // Create comments and attachments if present
    if let Some(comments_json) = &ticket_json.comments {
        for comment_json in comments_json {
            let created_at = NaiveDateTime::parse_from_str(&comment_json.created_at, "%Y-%m-%dT%H:%M:%S")
                .unwrap_or_else(|_| chrono::Local::now().naive_local());
            
            let new_comment = NewComment {
                content: comment_json.content.clone(),
                user_uuid: comment_json.user_uuid.clone(),
                created_at: Some(created_at),
                ticket_id: ticket.id,
            };
            
            let comment = create_comment(conn, new_comment)?;
            
            // Create attachments for this comment
            for attachment_json in &comment_json.attachments {
                let new_attachment = NewAttachment {
                    url: attachment_json.url.clone(),
                    name: attachment_json.name.clone(),
                    comment_id: Some(comment.id),
                };
                
                create_attachment(conn, new_attachment)?;
            }
        }
    }
    
    // Create article content if present
    if let Some(content) = &ticket_json.article_content {
        let new_article_content = NewArticleContent {
            content: content.clone(),
            ticket_id: ticket.id,
        };
        
        create_article_content(conn, new_article_content)?;
    }
    
    Ok(ticket)
}

// Document updates
use chrono::Utc;
use crate::models::{DocumentUpdate, NewDocumentUpdate};
use crate::schema::document_updates;

pub fn store_document_update(
    conn: &mut DbConnection,
    document_id: String,
    update_data: Vec<u8>,
    client_id: String,
) -> Result<DocumentUpdate, diesel::result::Error> {
    use diesel::prelude::*;
    use chrono::Utc;
    
    let new_update = NewDocumentUpdate {
        document_id,
        update_data,
        client_id,
    };
    
    diesel::insert_into(document_updates::table)
        .values(&new_update)
        .get_result(conn)
}

pub fn get_latest_document_update(
    conn: &mut DbConnection,
    doc_id: &str,
) -> Result<DocumentUpdate, diesel::result::Error> {
    use diesel::prelude::*;
    
    document_updates::table
        .filter(document_updates::document_id.eq(doc_id))
        .order(document_updates::created_at.desc())
        .first(conn)
}

pub fn get_document_updates(
    conn: &mut DbConnection,
    doc_id: &str,
    limit: i64,
) -> Result<Vec<DocumentUpdate>, diesel::result::Error> {
    use diesel::prelude::*;
    
    document_updates::table
        .filter(document_updates::document_id.eq(doc_id))
        .order(document_updates::created_at.desc())
        .limit(limit)
        .load::<DocumentUpdate>(conn)
} 