use diesel::prelude::*;
use diesel::result::Error;

use crate::db::DbConnection;
use crate::models::*;
use crate::schema::*;

// Document updates
pub fn store_document_update(
    conn: &mut DbConnection,
    document_id: String,
    update_data: Vec<u8>,
    client_id: String,
) -> Result<DocumentUpdate, Error> {
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
) -> Result<DocumentUpdate, Error> {
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
) -> Result<Vec<DocumentUpdate>, Error> {
    use diesel::prelude::*;
    
    document_updates::table
        .filter(document_updates::document_id.eq(doc_id))
        .order(document_updates::created_at.desc())
        .limit(limit)
        .load::<DocumentUpdate>(conn)
} 