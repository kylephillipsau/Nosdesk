use diesel::prelude::*;
use diesel::result::Error;
use diesel::sql_types::{Integer, Nullable};
use chrono::NaiveDateTime;

use crate::db::DbConnection;
use crate::models::{
    DocumentationPage, DocumentationPageWithChildren,
    NewDocumentationPage, PageOrder
};
use crate::schema::documentation_pages;

// Get all documentation pages
pub fn get_documentation_pages(conn: &mut DbConnection) -> Result<Vec<DocumentationPage>, Error> {
    documentation_pages::table
        .order_by(documentation_pages::title.asc())
        .load::<DocumentationPage>(conn)
}

// Get a specific documentation page by ID
pub fn get_documentation_page(id: i32, conn: &mut DbConnection) -> Result<DocumentationPage, Error> {
    documentation_pages::table
        .find(id)
        .first::<DocumentationPage>(conn)
}

// Get a documentation page by its slug
pub fn get_documentation_page_by_slug(slug: &str, conn: &mut DbConnection) -> Result<DocumentationPage, Error> {
    documentation_pages::table
        .filter(documentation_pages::slug.eq(slug))
        .first::<DocumentationPage>(conn)
}

// Create a new documentation page
pub fn create_documentation_page(
    page: NewDocumentationPage,
    conn: &mut DbConnection,
) -> Result<DocumentationPage, Error> {
    diesel::insert_into(documentation_pages::table)
        .values(page)
        .get_result(conn)
}

// Update an existing documentation page
pub fn update_documentation_page(
    conn: &mut DbConnection,
    page: &DocumentationPage,
) -> Result<DocumentationPage, Error> {
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

// Delete a documentation page
pub fn delete_documentation_page(id: i32, conn: &mut DbConnection) -> Result<usize, Error> {
    diesel::delete(documentation_pages::table.find(id)).execute(conn)
}

// Get top-level documentation pages
pub fn get_top_level_pages(conn: &mut DbConnection) -> Result<Vec<DocumentationPage>, Error> {
    documentation_pages::table
        .filter(documentation_pages::parent_id.is_null())
        .order_by(documentation_pages::title.asc())
        .load::<DocumentationPage>(conn)
}

// Get documentation pages by parent ID
pub fn get_pages_by_parent_id(parent_id: i32, conn: &mut DbConnection) -> Result<Vec<DocumentationPage>, Error> {
    documentation_pages::table
        .filter(documentation_pages::parent_id.eq(parent_id))
        .order_by(documentation_pages::title.asc())
        .load::<DocumentationPage>(conn)
}

// Get documentation pages by ticket ID
pub fn get_documentation_pages_by_ticket_id(conn: &mut DbConnection, ticket_id: i32) -> Result<Vec<DocumentationPage>, Error> {
    documentation_pages::table
        .filter(documentation_pages::ticket_id.eq(ticket_id))
        .order_by(documentation_pages::title.asc())
        .load::<DocumentationPage>(conn)
}

// Define a SQL function for coalesce
sql_function! {
    fn coalesce(x: Nullable<Integer>, y: Integer) -> Integer;
}

// Get ordered top-level documentation pages
pub fn get_ordered_top_level_pages(
    conn: &mut DbConnection,
) -> Result<Vec<DocumentationPage>, Error> {
    documentation_pages::table
        .filter(documentation_pages::parent_id.is_null())
        .order_by(coalesce(documentation_pages::display_order, 0).asc())
        .load::<DocumentationPage>(conn)
}

// Get ordered documentation pages by parent ID
pub fn get_ordered_pages_by_parent_id(
    conn: &mut DbConnection,
    parent_id: i32,
) -> Result<Vec<DocumentationPage>, Error> {
    documentation_pages::table
        .filter(documentation_pages::parent_id.eq(parent_id))
        .order_by(coalesce(documentation_pages::display_order, 0).asc())
        .load::<DocumentationPage>(conn)
}

// Reorder documentation pages
pub fn reorder_pages(
    conn: &mut DbConnection,
    parent_id: Option<i32>,
    page_orders: &[PageOrder],
) -> Result<Vec<DocumentationPage>, Error> {
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
    conn: &mut DbConnection,
    page_id: i32,
    new_parent_id: Option<i32>,
    display_order: i32,
) -> Result<DocumentationPage, Error> {
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
    conn: &mut DbConnection,
    page_id: i32,
) -> Result<DocumentationPageWithChildren, Error> {
    let page = get_documentation_page(page_id, conn)?;
    let children = get_ordered_pages_by_parent_id(conn, page_id)?;
    
    Ok(DocumentationPageWithChildren {
        page,
        children,
    })
}
