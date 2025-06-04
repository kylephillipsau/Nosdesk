use diesel::prelude::*;
use diesel::result::Error;
use diesel::QueryResult;

use crate::db::DbConnection;
use crate::models::*;
use crate::schema::*;

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
            start_date: project.start_date,
            end_date: project.end_date,
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
        start_date: project.start_date,
        end_date: project.end_date,
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
    match crate::repository::tickets::get_ticket_by_id(conn, ticket_id) {
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

// Get projects for a ticket
pub fn get_projects_for_ticket(conn: &mut DbConnection, ticket_id: i32) -> QueryResult<Vec<Project>> {
    println!("Getting projects for ticket ID: {}", ticket_id);
    
    project_tickets::table
        .filter(project_tickets::ticket_id.eq(ticket_id))
        .inner_join(projects::table)
        .select(projects::all_columns)
        .load::<Project>(conn)
} 