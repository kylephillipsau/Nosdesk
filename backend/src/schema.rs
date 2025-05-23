// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "documentation_status"))]
    pub struct DocumentationStatus;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "project_status"))]
    pub struct ProjectStatus;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "ticket_priority"))]
    pub struct TicketPriority;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "ticket_status"))]
    pub struct TicketStatus;
}

diesel::table! {
    article_contents (id) {
        id -> Int4,
        content -> Bytea,
        ticket_id -> Nullable<Int4>,
    }
}

diesel::table! {
    attachments (id) {
        id -> Int4,
        #[max_length = 255]
        url -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        comment_id -> Nullable<Int4>,
    }
}

diesel::table! {
    auth_providers (id) {
        id -> Int4,
        #[max_length = 50]
        provider_type -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        enabled -> Bool,
        is_default -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    comments (id) {
        id -> Int4,
        content -> Text,
        created_at -> Timestamp,
        #[max_length = 36]
        user_uuid -> Varchar,
        ticket_id -> Int4,
    }
}

diesel::table! {
    devices (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        hostname -> Varchar,
        #[max_length = 255]
        serial_number -> Varchar,
        #[max_length = 255]
        model -> Varchar,
        #[max_length = 50]
        warranty_status -> Varchar,
        ticket_id -> Nullable<Int4>,
    }
}

diesel::table! {
    document_updates (id) {
        id -> Int4,
        #[max_length = 255]
        document_id -> Varchar,
        update_data -> Bytea,
        #[max_length = 255]
        client_id -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::DocumentationStatus;

    documentation_pages (id) {
        id -> Int4,
        #[max_length = 255]
        slug -> Varchar,
        #[max_length = 255]
        title -> Varchar,
        description -> Nullable<Text>,
        content -> Bytea,
        #[max_length = 255]
        author -> Varchar,
        status -> DocumentationStatus,
        #[max_length = 50]
        icon -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        parent_id -> Nullable<Int4>,
        ticket_id -> Nullable<Int4>,
        display_order -> Nullable<Int4>,
    }
}

diesel::table! {
    linked_tickets (ticket_id, linked_ticket_id) {
        ticket_id -> Int4,
        linked_ticket_id -> Int4,
        created_at -> Timestamp,
    }
}

diesel::table! {
    project_tickets (project_id, ticket_id) {
        project_id -> Int4,
        ticket_id -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ProjectStatus;

    projects (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        description -> Nullable<Text>,
        status -> ProjectStatus,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TicketStatus;
    use super::sql_types::TicketPriority;

    tickets (id) {
        id -> Int4,
        #[max_length = 255]
        title -> Varchar,
        status -> TicketStatus,
        priority -> TicketPriority,
        created -> Timestamp,
        modified -> Timestamp,
        #[max_length = 255]
        assignee -> Nullable<Varchar>,
        #[max_length = 255]
        requester -> Nullable<Varchar>,
        closed_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    user_auth_identities (id) {
        id -> Int4,
        user_id -> Int4,
        auth_provider_id -> Int4,
        #[max_length = 255]
        provider_user_id -> Varchar,
        #[max_length = 255]
        email -> Nullable<Varchar>,
        identity_data -> Nullable<Jsonb>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        password_hash -> Nullable<Bytea>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 36]
        uuid -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 50]
        role -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        #[max_length = 50]
        pronouns -> Nullable<Varchar>,
        #[max_length = 255]
        avatar_url -> Nullable<Varchar>,
        #[max_length = 255]
        banner_url -> Nullable<Varchar>,
    }
}

diesel::joinable!(article_contents -> tickets (ticket_id));
diesel::joinable!(attachments -> comments (comment_id));
diesel::joinable!(comments -> tickets (ticket_id));
diesel::joinable!(devices -> tickets (ticket_id));
diesel::joinable!(documentation_pages -> tickets (ticket_id));
diesel::joinable!(project_tickets -> projects (project_id));
diesel::joinable!(project_tickets -> tickets (ticket_id));
diesel::joinable!(user_auth_identities -> auth_providers (auth_provider_id));
diesel::joinable!(user_auth_identities -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    article_contents,
    attachments,
    auth_providers,
    comments,
    devices,
    document_updates,
    documentation_pages,
    linked_tickets,
    project_tickets,
    projects,
    tickets,
    user_auth_identities,
    users,
);
