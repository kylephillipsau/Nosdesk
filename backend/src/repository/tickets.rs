use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::QueryResult;

use crate::db::DbConnection;
use crate::models::*;
use crate::schema::*;

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

// Composite operations for tickets
pub fn get_complete_ticket(conn: &mut DbConnection, ticket_id: i32) -> Result<CompleteTicket, Error> {
    println!("Getting complete ticket data for ticket ID: {}", ticket_id);
    
    let ticket = get_ticket_by_id(conn, ticket_id)?;
    println!("Found ticket: {} - {}", ticket.id, ticket.title);
    
    // Get device for this ticket
    let device = crate::repository::devices::get_device_by_ticket_id(conn, ticket_id).ok();
    
    // Get comments for this ticket
    let comments = crate::repository::comments::get_comments_by_ticket_id(conn, ticket_id)?;
    let mut comments_with_attachments = Vec::new();
    
    for comment in comments {
        let attachments = crate::repository::comments::get_attachments_by_comment_id(conn, comment.id)?;
        
        // Get user information for this comment
        let user = match crate::repository::users::get_user_by_uuid(&comment.user_uuid, conn) {
            Ok(user) => Some(UserInfo::from(user)),
            Err(_) => None,
        };
        
        comments_with_attachments.push(CommentWithAttachments {
            comment,
            attachments,
            user,
        });
    }
    
    // Get article content
    let article_content = crate::repository::article_content::get_article_content_by_ticket_id(conn, ticket_id)
        .map(|content| content.content)
        .ok();
    
    // Get linked tickets
    let linked_tickets = crate::repository::linked_tickets::get_linked_tickets(conn, ticket_id).unwrap_or_default();
    println!("Linked tickets for ticket {}: {:?}", ticket_id, linked_tickets);
    
    // Get projects for this ticket
    let projects = crate::repository::projects::get_projects_for_ticket(conn, ticket_id).unwrap_or_default();
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
        
        crate::repository::devices::create_device(conn, new_device)?;
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
            
            let comment = crate::repository::comments::create_comment(conn, new_comment)?;
            
            // Create attachments for this comment
            for attachment_json in &comment_json.attachments {
                let new_attachment = NewAttachment {
                    url: attachment_json.url.clone(),
                    name: attachment_json.name.clone(),
                    comment_id: Some(comment.id),
                };
                
                crate::repository::comments::create_attachment(conn, new_attachment)?;
            }
        }
    }
    
    // Create article content if present
    if let Some(content) = &ticket_json.article_content {
        let new_article_content = NewArticleContent {
            content: content.clone(),
            ticket_id: ticket.id,
        };
        
        crate::repository::article_content::create_article_content(conn, new_article_content)?;
    }
    
    Ok(ticket)
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
        
        crate::repository::devices::create_device(conn, new_device)?;
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
            
            let comment = crate::repository::comments::create_comment(conn, new_comment)?;
            
            // Create attachments for this comment
            for attachment_json in &comment_json.attachments {
                let new_attachment = NewAttachment {
                    url: attachment_json.url.clone(),
                    name: attachment_json.name.clone(),
                    comment_id: Some(comment.id),
                };
                
                crate::repository::comments::create_attachment(conn, new_attachment)?;
            }
        }
    }
    
    // Create article content if present
    if let Some(content) = &ticket_json.article_content {
        let new_article_content = NewArticleContent {
            content: content.clone(),
            ticket_id: ticket.id,
        };
        
        crate::repository::article_content::create_article_content(conn, new_article_content)?;
    }
    
    Ok(ticket)
} 