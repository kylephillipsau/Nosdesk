--
-- PostgreSQL database dump
--

-- Dumped from database version 16.8 (Homebrew)
-- Dumped by pg_dump version 16.8 (Homebrew)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: documentation_status; Type: TYPE; Schema: public; Owner: kylephillips
--

CREATE TYPE public.documentation_status AS ENUM (
    'draft',
    'published',
    'archived'
);


ALTER TYPE public.documentation_status OWNER TO kylephillips;

--
-- Name: project_status; Type: TYPE; Schema: public; Owner: kylephillips
--

CREATE TYPE public.project_status AS ENUM (
    'active',
    'completed',
    'archived'
);


ALTER TYPE public.project_status OWNER TO kylephillips;

--
-- Name: ticket_priority; Type: TYPE; Schema: public; Owner: kylephillips
--

CREATE TYPE public.ticket_priority AS ENUM (
    'low',
    'medium',
    'high'
);


ALTER TYPE public.ticket_priority OWNER TO kylephillips;

--
-- Name: ticket_status; Type: TYPE; Schema: public; Owner: kylephillips
--

CREATE TYPE public.ticket_status AS ENUM (
    'open',
    'in-progress',
    'closed'
);


ALTER TYPE public.ticket_status OWNER TO kylephillips;

--
-- Name: user_role; Type: TYPE; Schema: public; Owner: kylephillips
--

CREATE TYPE public.user_role AS ENUM (
    'admin',
    'technician',
    'user'
);


ALTER TYPE public.user_role OWNER TO kylephillips;

--
-- Name: diesel_manage_updated_at(regclass); Type: FUNCTION; Schema: public; Owner: kylephillips
--

CREATE FUNCTION public.diesel_manage_updated_at(_tbl regclass) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$;


ALTER FUNCTION public.diesel_manage_updated_at(_tbl regclass) OWNER TO kylephillips;

--
-- Name: diesel_set_updated_at(); Type: FUNCTION; Schema: public; Owner: kylephillips
--

CREATE FUNCTION public.diesel_set_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.diesel_set_updated_at() OWNER TO kylephillips;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: __diesel_schema_migrations; Type: TABLE; Schema: public; Owner: kylephillips
--

CREATE TABLE public.__diesel_schema_migrations (
    version character varying(50) NOT NULL,
    run_on timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public.__diesel_schema_migrations OWNER TO kylephillips;

--
-- Name: article_contents; Type: TABLE; Schema: public; Owner: kylephillips
--

CREATE TABLE public.article_contents (
    id integer NOT NULL,
    content text NOT NULL,
    ticket_id integer
);


ALTER TABLE public.article_contents OWNER TO kylephillips;

--
-- Name: article_contents_id_seq; Type: SEQUENCE; Schema: public; Owner: kylephillips
--

CREATE SEQUENCE public.article_contents_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.article_contents_id_seq OWNER TO kylephillips;

--
-- Name: article_contents_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: kylephillips
--

ALTER SEQUENCE public.article_contents_id_seq OWNED BY public.article_contents.id;


--
-- Name: attachments; Type: TABLE; Schema: public; Owner: kylephillips
--

CREATE TABLE public.attachments (
    id integer NOT NULL,
    url character varying(255) NOT NULL,
    name character varying(255) NOT NULL,
    comment_id integer
);


ALTER TABLE public.attachments OWNER TO kylephillips;

--
-- Name: attachments_id_seq; Type: SEQUENCE; Schema: public; Owner: kylephillips
--

CREATE SEQUENCE public.attachments_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.attachments_id_seq OWNER TO kylephillips;

--
-- Name: attachments_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: kylephillips
--

ALTER SEQUENCE public.attachments_id_seq OWNED BY public.attachments.id;


--
-- Name: comments; Type: TABLE; Schema: public; Owner: kylephillips
--

CREATE TABLE public.comments (
    id integer NOT NULL,
    content text NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    user_uuid character varying(36) NOT NULL,
    ticket_id integer NOT NULL
);


ALTER TABLE public.comments OWNER TO kylephillips;

--
-- Name: comments_id_seq; Type: SEQUENCE; Schema: public; Owner: kylephillips
--

CREATE SEQUENCE public.comments_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.comments_id_seq OWNER TO kylephillips;

--
-- Name: comments_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: kylephillips
--

ALTER SEQUENCE public.comments_id_seq OWNED BY public.comments.id;


--
-- Name: devices; Type: TABLE; Schema: public; Owner: kylephillips
--

CREATE TABLE public.devices (
    id integer NOT NULL,
    name character varying(255) NOT NULL,
    hostname character varying(255) NOT NULL,
    serial_number character varying(255) NOT NULL,
    model character varying(255) NOT NULL,
    warranty_status character varying(50) NOT NULL,
    ticket_id integer
);


ALTER TABLE public.devices OWNER TO kylephillips;

--
-- Name: devices_id_seq; Type: SEQUENCE; Schema: public; Owner: kylephillips
--

CREATE SEQUENCE public.devices_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.devices_id_seq OWNER TO kylephillips;

--
-- Name: devices_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: kylephillips
--

ALTER SEQUENCE public.devices_id_seq OWNED BY public.devices.id;


--
-- Name: documentation_pages; Type: TABLE; Schema: public; Owner: kylephillips
--

CREATE TABLE public.documentation_pages (
    id integer NOT NULL,
    slug character varying(255) NOT NULL,
    title character varying(255) NOT NULL,
    description text,
    content text NOT NULL,
    author character varying(255) NOT NULL,
    status public.documentation_status DEFAULT 'draft'::public.documentation_status NOT NULL,
    icon character varying(50),
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL,
    parent_id integer,
    ticket_id integer,
    display_order integer DEFAULT 0
);


ALTER TABLE public.documentation_pages OWNER TO kylephillips;

--
-- Name: documentation_pages_id_seq; Type: SEQUENCE; Schema: public; Owner: kylephillips
--

CREATE SEQUENCE public.documentation_pages_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.documentation_pages_id_seq OWNER TO kylephillips;

--
-- Name: documentation_pages_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: kylephillips
--

ALTER SEQUENCE public.documentation_pages_id_seq OWNED BY public.documentation_pages.id;


--
-- Name: linked_tickets; Type: TABLE; Schema: public; Owner: kylephillips
--

CREATE TABLE public.linked_tickets (
    ticket_id integer NOT NULL,
    linked_ticket_id integer NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT no_self_link CHECK ((ticket_id <> linked_ticket_id))
);


ALTER TABLE public.linked_tickets OWNER TO kylephillips;

--
-- Name: project_tickets; Type: TABLE; Schema: public; Owner: kylephillips
--

CREATE TABLE public.project_tickets (
    project_id integer NOT NULL,
    ticket_id integer NOT NULL
);


ALTER TABLE public.project_tickets OWNER TO kylephillips;

--
-- Name: projects; Type: TABLE; Schema: public; Owner: kylephillips
--

CREATE TABLE public.projects (
    id integer NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    status public.project_status DEFAULT 'active'::public.project_status NOT NULL,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.projects OWNER TO kylephillips;

--
-- Name: projects_id_seq; Type: SEQUENCE; Schema: public; Owner: kylephillips
--

CREATE SEQUENCE public.projects_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.projects_id_seq OWNER TO kylephillips;

--
-- Name: projects_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: kylephillips
--

ALTER SEQUENCE public.projects_id_seq OWNED BY public.projects.id;


--
-- Name: tickets; Type: TABLE; Schema: public; Owner: kylephillips
--

CREATE TABLE public.tickets (
    id integer NOT NULL,
    title character varying(255) NOT NULL,
    status public.ticket_status DEFAULT 'open'::public.ticket_status NOT NULL,
    priority public.ticket_priority DEFAULT 'medium'::public.ticket_priority NOT NULL,
    created timestamp without time zone DEFAULT CURRENT_DATE NOT NULL,
    modified timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    assignee character varying(255),
    requester character varying(255),
    closed_at timestamp without time zone
);


ALTER TABLE public.tickets OWNER TO kylephillips;

--
-- Name: tickets_id_seq; Type: SEQUENCE; Schema: public; Owner: kylephillips
--

CREATE SEQUENCE public.tickets_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.tickets_id_seq OWNER TO kylephillips;

--
-- Name: tickets_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: kylephillips
--

ALTER SEQUENCE public.tickets_id_seq OWNED BY public.tickets.id;


--
-- Name: users; Type: TABLE; Schema: public; Owner: kylephillips
--

CREATE TABLE public.users (
    id integer NOT NULL,
    uuid character varying(36) NOT NULL,
    name character varying(255) NOT NULL,
    email character varying(255) NOT NULL,
    role character varying(50) DEFAULT 'user'::character varying NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    password_hash bytea DEFAULT '\x'::bytea NOT NULL
);


ALTER TABLE public.users OWNER TO kylephillips;

--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: kylephillips
--

CREATE SEQUENCE public.users_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.users_id_seq OWNER TO kylephillips;

--
-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: kylephillips
--

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;


--
-- Name: article_contents id; Type: DEFAULT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.article_contents ALTER COLUMN id SET DEFAULT nextval('public.article_contents_id_seq'::regclass);


--
-- Name: attachments id; Type: DEFAULT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.attachments ALTER COLUMN id SET DEFAULT nextval('public.attachments_id_seq'::regclass);


--
-- Name: comments id; Type: DEFAULT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.comments ALTER COLUMN id SET DEFAULT nextval('public.comments_id_seq'::regclass);


--
-- Name: devices id; Type: DEFAULT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.devices ALTER COLUMN id SET DEFAULT nextval('public.devices_id_seq'::regclass);


--
-- Name: documentation_pages id; Type: DEFAULT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.documentation_pages ALTER COLUMN id SET DEFAULT nextval('public.documentation_pages_id_seq'::regclass);


--
-- Name: projects id; Type: DEFAULT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.projects ALTER COLUMN id SET DEFAULT nextval('public.projects_id_seq'::regclass);


--
-- Name: tickets id; Type: DEFAULT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.tickets ALTER COLUMN id SET DEFAULT nextval('public.tickets_id_seq'::regclass);


--
-- Name: users id; Type: DEFAULT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass);


--
-- Name: __diesel_schema_migrations __diesel_schema_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.__diesel_schema_migrations
    ADD CONSTRAINT __diesel_schema_migrations_pkey PRIMARY KEY (version);


--
-- Name: article_contents article_contents_pkey; Type: CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.article_contents
    ADD CONSTRAINT article_contents_pkey PRIMARY KEY (id);


--
-- Name: attachments attachments_pkey; Type: CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.attachments
    ADD CONSTRAINT attachments_pkey PRIMARY KEY (id);


--
-- Name: comments comments_pkey; Type: CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_pkey PRIMARY KEY (id);


--
-- Name: devices devices_pkey; Type: CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.devices
    ADD CONSTRAINT devices_pkey PRIMARY KEY (id);


--
-- Name: documentation_pages documentation_pages_pkey; Type: CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.documentation_pages
    ADD CONSTRAINT documentation_pages_pkey PRIMARY KEY (id);


--
-- Name: documentation_pages documentation_pages_slug_key; Type: CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.documentation_pages
    ADD CONSTRAINT documentation_pages_slug_key UNIQUE (slug);


--
-- Name: linked_tickets linked_tickets_pkey; Type: CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.linked_tickets
    ADD CONSTRAINT linked_tickets_pkey PRIMARY KEY (ticket_id, linked_ticket_id);


--
-- Name: project_tickets project_tickets_pkey; Type: CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.project_tickets
    ADD CONSTRAINT project_tickets_pkey PRIMARY KEY (project_id, ticket_id);


--
-- Name: projects projects_pkey; Type: CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.projects
    ADD CONSTRAINT projects_pkey PRIMARY KEY (id);


--
-- Name: tickets tickets_pkey; Type: CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.tickets
    ADD CONSTRAINT tickets_pkey PRIMARY KEY (id);


--
-- Name: article_contents unique_ticket_article; Type: CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.article_contents
    ADD CONSTRAINT unique_ticket_article UNIQUE (ticket_id);


--
-- Name: users users_email_key; Type: CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_email_key UNIQUE (email);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: users users_uuid_key; Type: CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_uuid_key UNIQUE (uuid);


--
-- Name: idx_documentation_pages_display_order; Type: INDEX; Schema: public; Owner: kylephillips
--

CREATE INDEX idx_documentation_pages_display_order ON public.documentation_pages USING btree (display_order);


--
-- Name: idx_documentation_pages_parent_id; Type: INDEX; Schema: public; Owner: kylephillips
--

CREATE INDEX idx_documentation_pages_parent_id ON public.documentation_pages USING btree (parent_id);


--
-- Name: idx_documentation_pages_ticket_id; Type: INDEX; Schema: public; Owner: kylephillips
--

CREATE INDEX idx_documentation_pages_ticket_id ON public.documentation_pages USING btree (ticket_id);


--
-- Name: idx_linked_tickets_linked_ticket_id; Type: INDEX; Schema: public; Owner: kylephillips
--

CREATE INDEX idx_linked_tickets_linked_ticket_id ON public.linked_tickets USING btree (linked_ticket_id);


--
-- Name: idx_linked_tickets_ticket_id; Type: INDEX; Schema: public; Owner: kylephillips
--

CREATE INDEX idx_linked_tickets_ticket_id ON public.linked_tickets USING btree (ticket_id);


--
-- Name: idx_project_tickets_project_id; Type: INDEX; Schema: public; Owner: kylephillips
--

CREATE INDEX idx_project_tickets_project_id ON public.project_tickets USING btree (project_id);


--
-- Name: idx_project_tickets_ticket_id; Type: INDEX; Schema: public; Owner: kylephillips
--

CREATE INDEX idx_project_tickets_ticket_id ON public.project_tickets USING btree (ticket_id);


--
-- Name: idx_users_email; Type: INDEX; Schema: public; Owner: kylephillips
--

CREATE INDEX idx_users_email ON public.users USING btree (email);


--
-- Name: idx_users_uuid; Type: INDEX; Schema: public; Owner: kylephillips
--

CREATE INDEX idx_users_uuid ON public.users USING btree (uuid);


--
-- Name: article_contents article_contents_ticket_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.article_contents
    ADD CONSTRAINT article_contents_ticket_id_fkey FOREIGN KEY (ticket_id) REFERENCES public.tickets(id) ON DELETE CASCADE;


--
-- Name: attachments attachments_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.attachments
    ADD CONSTRAINT attachments_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comments(id);


--
-- Name: comments comments_ticket_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_ticket_id_fkey FOREIGN KEY (ticket_id) REFERENCES public.tickets(id);


--
-- Name: comments comments_user_uuid_fkey; Type: FK CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_user_uuid_fkey FOREIGN KEY (user_uuid) REFERENCES public.users(uuid);


--
-- Name: devices devices_ticket_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.devices
    ADD CONSTRAINT devices_ticket_id_fkey FOREIGN KEY (ticket_id) REFERENCES public.tickets(id) ON DELETE CASCADE;


--
-- Name: documentation_pages documentation_pages_parent_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.documentation_pages
    ADD CONSTRAINT documentation_pages_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES public.documentation_pages(id);


--
-- Name: documentation_pages fk_documentation_pages_ticket; Type: FK CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.documentation_pages
    ADD CONSTRAINT fk_documentation_pages_ticket FOREIGN KEY (ticket_id) REFERENCES public.tickets(id) ON DELETE SET NULL;


--
-- Name: linked_tickets linked_tickets_linked_ticket_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.linked_tickets
    ADD CONSTRAINT linked_tickets_linked_ticket_id_fkey FOREIGN KEY (linked_ticket_id) REFERENCES public.tickets(id) ON DELETE CASCADE;


--
-- Name: linked_tickets linked_tickets_ticket_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.linked_tickets
    ADD CONSTRAINT linked_tickets_ticket_id_fkey FOREIGN KEY (ticket_id) REFERENCES public.tickets(id) ON DELETE CASCADE;


--
-- Name: project_tickets project_tickets_project_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.project_tickets
    ADD CONSTRAINT project_tickets_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;


--
-- Name: project_tickets project_tickets_ticket_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: kylephillips
--

ALTER TABLE ONLY public.project_tickets
    ADD CONSTRAINT project_tickets_ticket_id_fkey FOREIGN KEY (ticket_id) REFERENCES public.tickets(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

