use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::QueryResult;
use uuid::Uuid;

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
    assignee: Option<String>,
    created_after: Option<String>,
    created_before: Option<String>,
    created_on: Option<String>,
    modified_after: Option<String>,
    modified_before: Option<String>,
    modified_on: Option<String>,
    closed_after: Option<String>,
    closed_before: Option<String>,
    closed_on: Option<String>,
) -> Result<(Vec<Ticket>, i64), Error> {
    // Build the main query
    let mut query = tickets::table.into_boxed();
    
    // Apply filters if provided
    if let Some(search_term) = search.clone() {
        if !search_term.is_empty() {
            let search_pattern = format!("%{}%", search_term.to_lowercase());
            query = query.filter(
                tickets::title.ilike(search_pattern.clone())
                    .or(tickets::description.ilike(search_pattern.clone()))
                    .or(tickets::id.eq_any(
                        search_term.parse::<i32>().ok().map(|id| vec![id]).unwrap_or_default()
                    ))
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
                    .or(tickets::description.ilike(search_pattern.clone()))
                    .or(tickets::id.eq_any(
                        search_term.parse::<i32>().ok().map(|id| vec![id]).unwrap_or_default()
                    ))
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
    
    // Handle assignee filter for count query  
    if let Some(assignee_filter) = &assignee {
        if assignee_filter != "all" {
            // Parse the UUID string
            if let Ok(assignee_uuid) = uuid::Uuid::parse_str(assignee_filter) {
                count_query = count_query.filter(tickets::assignee_uuid.eq(Some(assignee_uuid)));
                query = query.filter(tickets::assignee_uuid.eq(Some(assignee_uuid)));
            }
        }
    }
    
    // Handle date filtering
    // Created date filters
    if let Some(created_after_str) = &created_after {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(created_after_str, "%Y-%m-%d") {
            let datetime = date.and_hms_opt(0, 0, 0).unwrap();
            query = query.filter(tickets::created_at.ge(datetime));
            count_query = count_query.filter(tickets::created_at.ge(datetime));
        }
    }
    
    if let Some(created_before_str) = &created_before {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(created_before_str, "%Y-%m-%d") {
            let datetime = date.and_hms_opt(23, 59, 59).unwrap();
            query = query.filter(tickets::created_at.le(datetime));
            count_query = count_query.filter(tickets::created_at.le(datetime));
        }
    }
    
    if let Some(created_on_str) = &created_on {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(created_on_str, "%Y-%m-%d") {
            let start_datetime = date.and_hms_opt(0, 0, 0).unwrap();
            let end_datetime = date.and_hms_opt(23, 59, 59).unwrap();
            query = query.filter(tickets::created_at.between(start_datetime, end_datetime));
            count_query = count_query.filter(tickets::created_at.between(start_datetime, end_datetime));
        }
    }
    
    // Modified date filters (using updated_at column)
    if let Some(modified_after_str) = &modified_after {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(modified_after_str, "%Y-%m-%d") {
            let datetime = date.and_hms_opt(0, 0, 0).unwrap();
            query = query.filter(tickets::updated_at.ge(datetime));
            count_query = count_query.filter(tickets::updated_at.ge(datetime));
        }
    }
    
    if let Some(modified_before_str) = &modified_before {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(modified_before_str, "%Y-%m-%d") {
            let datetime = date.and_hms_opt(23, 59, 59).unwrap();
            query = query.filter(tickets::updated_at.le(datetime));
            count_query = count_query.filter(tickets::updated_at.le(datetime));
        }
    }
    
    if let Some(modified_on_str) = &modified_on {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(modified_on_str, "%Y-%m-%d") {
            let start_datetime = date.and_hms_opt(0, 0, 0).unwrap();
            let end_datetime = date.and_hms_opt(23, 59, 59).unwrap();
            query = query.filter(tickets::updated_at.between(start_datetime, end_datetime));
            count_query = count_query.filter(tickets::updated_at.between(start_datetime, end_datetime));
        }
    }
    
    // Closed date filtering
    if let Some(closed_after_str) = &closed_after {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(closed_after_str, "%Y-%m-%d") {
            let start_datetime = date.and_hms_opt(0, 0, 0).unwrap();
            query = query.filter(tickets::closed_at.gt(start_datetime));
            count_query = count_query.filter(tickets::closed_at.gt(start_datetime));
        }
    }
    
    if let Some(closed_before_str) = &closed_before {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(closed_before_str, "%Y-%m-%d") {
            let end_datetime = date.and_hms_opt(23, 59, 59).unwrap();
            query = query.filter(tickets::closed_at.lt(end_datetime));
            count_query = count_query.filter(tickets::closed_at.lt(end_datetime));
        }
    }
    
    if let Some(closed_on_str) = &closed_on {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(closed_on_str, "%Y-%m-%d") {
            let start_datetime = date.and_hms_opt(0, 0, 0).unwrap();
            let end_datetime = date.and_hms_opt(23, 59, 59).unwrap();
            query = query.filter(tickets::closed_at.between(start_datetime, end_datetime));
            count_query = count_query.filter(tickets::closed_at.between(start_datetime, end_datetime));
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
        (Some("created_at"), Some("asc")) => query = query.order(tickets::created_at.asc()),
        (Some("created_at"), _) => query = query.order(tickets::created_at.desc()),
        (Some("requester_uuid"), Some("asc")) => query = query.order(tickets::requester_uuid.asc()),
        (Some("requester_uuid"), _) => query = query.order(tickets::requester_uuid.desc()),
        (Some("assignee_uuid"), Some("asc")) => query = query.order(tickets::assignee_uuid.asc()),
        (Some("assignee_uuid"), _) => query = query.order(tickets::assignee_uuid.desc()),
        _ => query = query.order(tickets::id.desc()), // Default sort
    }
    
    // Apply pagination
    let offset = (page - 1) * page_size;
    query = query.offset(offset).limit(page_size);
    
    // Execute the query
    let results = query.load::<Ticket>(conn)?;
    
    Ok((results, total))
}

// Get paginated tickets with user information for list views
pub fn get_paginated_tickets_with_users(
    conn: &mut DbConnection,
    page: i64,
    page_size: i64,
    sort_field: Option<String>,
    sort_direction: Option<String>,
    search: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    assignee: Option<String>,
    created_after: Option<String>,
    created_before: Option<String>,
    created_on: Option<String>,
    modified_after: Option<String>,
    modified_before: Option<String>,
    modified_on: Option<String>,
    closed_after: Option<String>,
    closed_before: Option<String>,
    closed_on: Option<String>,
) -> Result<(Vec<crate::models::TicketListItem>, i64), Error> {
    // First get the basic tickets and total count
    let (tickets, total) = get_paginated_tickets(
        conn, page, page_size, sort_field, sort_direction, 
        search, status, priority, assignee,
        created_after, created_before, created_on,
        modified_after, modified_before, modified_on,
        closed_after, closed_before, closed_on
    )?;

    // Convert to TicketListItem with user information
    let mut ticket_list_items = Vec::new();
    
    for ticket in tickets {
        // Look up complete user data for requester and assignee
        let requester_user = ticket.requester_uuid.as_ref()
            .and_then(|uuid| crate::repository::get_user_by_uuid(uuid, conn).ok())
            .map(crate::models::UserInfoWithAvatar::from);
        
        let assignee_user = match ticket.assignee_uuid {
            Some(assignee_uuid) => {
                match crate::repository::get_user_by_uuid(&assignee_uuid, conn) {
                    Ok(user) => Some(crate::models::UserInfoWithAvatar::from(user)),
                    Err(_) => None,
                }
            },
            None => None,
        };

        ticket_list_items.push(crate::models::TicketListItem {
            ticket,
            requester_user,
            assignee_user,
        });
    }

    Ok((ticket_list_items, total))
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
    // Get the main ticket first
    let ticket = get_ticket_by_id(conn, ticket_id)?;
    println!("Found ticket: {} - {}", ticket.id, ticket.title);
    
    // Look up complete user data for requester and assignee
    let requester_user = ticket.requester_uuid.as_ref()
        .and_then(|uuid| crate::repository::get_user_by_uuid(uuid, conn).ok())
        .map(crate::models::UserInfoWithAvatar::from);
    
    let assignee_user = match ticket.assignee_uuid {
        Some(assignee_uuid) => {
            match crate::repository::get_user_by_uuid(&assignee_uuid, conn) {
                Ok(user) => Some(UserInfoWithAvatar::from(user)),
                Err(_) => None, // User not found
            }
        },
        None => None, // No assignee
    };
    
    // Get devices associated with this ticket through the junction table
    let devices = get_devices_for_ticket(conn, ticket_id).unwrap_or_default();
    
    // Get comments for this ticket
    let comments = crate::repository::comments::get_comments_by_ticket_id(conn, ticket_id)?;
    let mut comments_with_attachments = Vec::new();
    
    for comment in comments {
        let attachments = crate::repository::comments::get_attachments_by_comment_id(conn, comment.id)?;
        
        // Get user information for this comment with avatar
        let user = match crate::repository::users::get_user_by_id(comment.user_id, conn) {
            Ok(user) => Some(UserInfoWithAvatar::from(user)),
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
        requester_user,
        assignee_user,
        devices,
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
        description: None, // No description field in TicketJson
        status,
        priority,
        requester_uuid: Some(Uuid::parse_str(&ticket_json.requester).unwrap_or_else(|_| Uuid::new_v4())),
        assignee_uuid: if ticket_json.assignee.is_empty() { 
            None 
        } else { 
            Uuid::parse_str(&ticket_json.assignee).ok()
        },
    };
    
    let ticket = create_ticket(conn, new_ticket)?;
    
    // Create device if present (without ticket association)
    if let Some(device_json) = &ticket_json.device {
        let new_device = NewDevice {
            name: device_json.name.clone(),
            hostname: Some(device_json.hostname.clone()),
            device_type: None,
            serial_number: Some(device_json.serial_number.clone()),
            manufacturer: None, // Will be populated during Microsoft Entra sync
            model: Some(device_json.model.clone()),
            warranty_status: Some(device_json.warranty_status.clone()),
            location: None,
            notes: None,
            user_id: None,
            primary_user_uuid: None, // Will be populated during Microsoft Entra sync
            azure_device_id: None,
            intune_device_id: None,
            entra_device_id: None,
            compliance_state: None,
            last_sync_time: None,
            operating_system: None,
            os_version: None,
            is_managed: None,
            enrollment_date: None,
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
                ticket_id: ticket.id,
                user_id: 1, // Default user ID since we don't have user mapping from UUID
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

#[allow(dead_code)]
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
        description: None, // No description field in TicketJson
        status,
        priority,
        requester_uuid: Some(Uuid::parse_str(&ticket_json.requester).unwrap_or_else(|_| Uuid::new_v4())),
        assignee_uuid: if ticket_json.assignee.is_empty() { 
            None 
        } else { 
            Uuid::parse_str(&ticket_json.assignee).ok()
        },
    };
    
    let ticket = create_ticket(conn, new_ticket)?;
    
    // Create device if present (without ticket association)
    if let Some(device_json) = &ticket_json.device {
        let new_device = NewDevice {
            name: device_json.name.clone(),
            hostname: Some(device_json.hostname.clone()),
            device_type: None,
            serial_number: Some(device_json.serial_number.clone()),
            manufacturer: None, // Will be populated during Microsoft Entra sync
            model: Some(device_json.model.clone()),
            warranty_status: Some(device_json.warranty_status.clone()),
            location: None,
            notes: None,
            user_id: None,
            primary_user_uuid: None, // Will be populated during Microsoft Entra sync
            azure_device_id: None,
            intune_device_id: None,
            entra_device_id: None,
            compliance_state: None,
            last_sync_time: None,
            operating_system: None,
            os_version: None,
            is_managed: None,
            enrollment_date: None,
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
                ticket_id: ticket.id,
                user_id: 1, // Default user ID since we don't have user mapping from UUID
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

// Ticket-Device relationship functions
pub fn add_device_to_ticket(conn: &mut DbConnection, ticket_id: i32, device_id: i32) -> QueryResult<TicketDevice> {
    let new_ticket_device = NewTicketDevice {
        ticket_id,
        device_id,
    };
    
    diesel::insert_into(ticket_devices::table)
        .values(&new_ticket_device)
        .get_result(conn)
}

pub fn remove_device_from_ticket(conn: &mut DbConnection, ticket_id: i32, device_id: i32) -> QueryResult<usize> {
    diesel::delete(
        ticket_devices::table
            .filter(ticket_devices::ticket_id.eq(ticket_id))
            .filter(ticket_devices::device_id.eq(device_id))
    ).execute(conn)
}

pub fn get_devices_for_ticket(conn: &mut DbConnection, ticket_id: i32) -> QueryResult<Vec<Device>> {
    ticket_devices::table
        .inner_join(devices::table)
        .filter(ticket_devices::ticket_id.eq(ticket_id))
        .select(devices::all_columns)
        .load(conn)
}

#[allow(dead_code)]
pub fn get_tickets_for_device(conn: &mut DbConnection, device_id: i32) -> QueryResult<Vec<Ticket>> {
    ticket_devices::table
        .inner_join(tickets::table)
        .filter(ticket_devices::device_id.eq(device_id))
        .select(tickets::all_columns)
        .load(conn)
} 