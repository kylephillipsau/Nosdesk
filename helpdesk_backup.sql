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
-- Data for Name: __diesel_schema_migrations; Type: TABLE DATA; Schema: public; Owner: kylephillips
--

COPY public.__diesel_schema_migrations (version, run_on) FROM stdin;
00000000000000	2025-02-27 05:04:26.230264
20250227103504	2025-02-27 10:35:58.933832
20250227232650	2025-02-27 23:27:56.117684
20250227235137	2025-02-27 23:52:26.188032
20250228064514	2025-02-28 06:45:48.6652
20250228070651	2025-03-02 02:54:54.389213
20250302025222	2025-03-02 02:54:54.390892
YYYYMMDDHHMMSS	2025-03-02 02:54:54.393221
20250302041743	2025-03-02 04:18:21.662594
20250302222106	2025-03-02 22:21:48.964827
20250303021542	2025-03-03 02:18:53.808424
20250303021828	2025-03-03 02:18:53.822677
20250303112610	2025-03-03 11:27:45.673744
20250304213411	2025-03-04 21:41:12.740654
20250305041444	2025-03-05 04:15:15.020056
20250305042734	2025-03-05 04:30:44.601845
20250306000000	2025-03-07 00:46:33.392964
20250307221112	2025-03-07 22:12:03.854575
20250307232724	2025-03-07 23:49:31.971631
\.


--
-- Data for Name: article_contents; Type: TABLE DATA; Schema: public; Owner: kylephillips
--

COPY public.article_contents (id, content, ticket_id) FROM stdin;
30	# This is my first Nosdesk ticket\n\nI am just testing the backend now, should be up and running.\n\nYou can add attachments and comments, and all that cool stuff\n\nThis is a collaborative markdown editor\n\n* You can see the updates in real time with other users	1
31	# Implement the ticket notes\n\nI have created a backend handler that should be able to take these notes from the \\`MarkdownEditor.vue\\` component\n\nWoo yeah baby, this is working flawlessly!	45
32	I am working on it, believe me lol\n\nIt is actually going pretty darn well and I am very happy with the progress that I am making on the application.	12
34	Chromebook was accidentally dropped in the bin after trying to install NAP Locked Down Browser on it.\n\n## Suggested resolutions\n\n* Leave it in the bin\n\n* Get an industrial shredder to finish the job	47
38	test	50
41	https://tidal.com/browse/album/388102216?u	53
35	Testing	15
42	# Headings\n\n* Dot point lists\n\nAll that stuff	54
\.


--
-- Data for Name: attachments; Type: TABLE DATA; Schema: public; Owner: kylephillips
--

COPY public.attachments (id, url, name, comment_id) FROM stdin;
22	/uploads/temp/a9dc8f73-a915-4244-85c0-7ca01594d7a3_Voice Note Mar 6, 01:19 PM.webm	Voice Note Mar 6, 01:19 PM.webm	\N
24	/uploads/tickets/1/6ca29b16-7ce3-4243-882f-7cb2b6364ca2_signal-2025-03-03-134946_028.jpeg	signal-2025-03-03-134946_028.jpeg	50
25	/uploads/tickets/54/851ad87a-b0cc-4d81-8fcb-2543af2aa111_IMG_5124.HEIC	IMG_5124.HEIC	52
26	/uploads/tickets/54/c86e92be-85a0-4c18-9283-9a382eb00b4c_Voice Note Mar 8, 04:00 PM.webm	Voice Note Mar 8, 04:00 PM.webm	53
\.


--
-- Data for Name: comments; Type: TABLE DATA; Schema: public; Owner: kylephillips
--

COPY public.comments (id, content, created_at, user_uuid, ticket_id) FROM stdin;
19	test	2025-03-05 05:23:14.257606	4c105c3b-9bce-4a7c-b93d-97783da1a192	45
20	I am testing the comments section	2025-03-05 05:23:20.117112	4c105c3b-9bce-4a7c-b93d-97783da1a192	45
33	You can add comments and stuff and all the most recent comments are at the top	2025-03-06 01:24:56.781641	4c105c3b-9bce-4a7c-b93d-97783da1a192	1
34	You can add new comments here and all the most recent comments display near the top	2025-03-06 01:25:48.35997	4c105c3b-9bce-4a7c-b93d-97783da1a192	12
35	So like this one here will display above that one that I put in here	2025-03-06 01:25:58.768159	4c105c3b-9bce-4a7c-b93d-97783da1a192	12
36	Honestly I would say that the backend is so good man	2025-03-06 01:26:10.629662	4c105c3b-9bce-4a7c-b93d-97783da1a192	12
37	It's written in Rust with a PostgreSQL server all set up and running to handle the stream of data that comes out of this application	2025-03-06 01:26:33.024243	4c105c3b-9bce-4a7c-b93d-97783da1a192	12
46	test	2025-03-06 02:26:52.672506	4c105c3b-9bce-4a7c-b93d-97783da1a192	50
50	Attachment added	2025-03-06 04:16:20.761065	4c105c3b-9bce-4a7c-b93d-97783da1a192	1
51	You can add comments	2025-03-08 04:59:36.971994	4c105c3b-9bce-4a7c-b93d-97783da1a192	54
52	Attachment added	2025-03-08 04:59:47.881225	4c105c3b-9bce-4a7c-b93d-97783da1a192	54
53	Attachment added	2025-03-08 05:00:12.786268	4c105c3b-9bce-4a7c-b93d-97783da1a192	54
\.


--
-- Data for Name: devices; Type: TABLE DATA; Schema: public; Owner: kylephillips
--

COPY public.devices (id, name, hostname, serial_number, model, warranty_status, ticket_id) FROM stdin;
1	L13G2-SN-00ACVP	L13G2-SN-00ACVP	PW00ACVP	20VLS0L100	Active	1
2	L13G2-SN-00ACVP	L13G2-SN-00ACVP	PW00ACVP	20VLS0L100	Active	8
\.


--
-- Data for Name: documentation_pages; Type: TABLE DATA; Schema: public; Owner: kylephillips
--

COPY public.documentation_pages (id, slug, title, description, content, author, status, icon, created_at, updated_at, parent_id, ticket_id, display_order) FROM stdin;
4	tickets	Ticket Management	Overview of Ticket Management	# Ticket Management\n\nThis section covers everything you need to know about creating, managing, and resolving support tickets in Nosdesk.\n\n## Key Ticket Features\n\n- Create and assign tickets\n- Set priorities and due dates\n- Track ticket status\n- Add comments and attachments\n- Link tickets to knowledge base articles\n\n## Best Practices\n\n- Use clear, descriptive titles\n- Include all relevant information in the description\n- Assign appropriate priority levels\n- Update ticket status as you work\n- Document resolution steps\n\nEffective ticket management is essential for providing excellent customer support and maintaining a record of issues and their resolutions.	System	published	🎫	2025-02-28 06:52:35.09213	2025-03-04 01:49:47.783844	\N	\N	1
1	getting-started	Getting Started	Overview of Getting Started	# Getting Started with Nosdesk\n\nThis section contains all the essential information to help you get up and running with Nosdesk quickly.\n\n## What You'll Find Here\n\n* Introduction to the platform\n\n* Quick start guides\n\n* Basic configuration steps\n\n* First-time user tutorials\n\n## Recommended First Steps\n\n1. Read the Welcome guide\n\n2. Follow the Quick Start guide\n\n3. Set up your user profile\n\n4. Configure your workspace preferences	System Admin	published	🚀	2025-02-28 06:52:35.088357	2025-03-05 03:06:35.895839	2	\N	0
2	welcome	Welcome to Nosdesk	Introduction to the helpdesk system	# Welcome to Nosdesk\n\nThis is the introduction to our helpdesk system. Nosdesk is designed to streamline your support operations and provide a seamless experience for both support agents and customers.\n\n## Key Features\n\n* **Ticket Management**: Create, assign, and track support tickets\n\n* **Knowledge Base**: Create and maintain documentation\n\n* **Asset Management**: Track devices and software\n\n* **Team Collaboration**: Work together seamlessly\n\n## Getting Started\n\n1. Create your account\n\n2. Set up your team\n\n3. Configure workflows\n\n4. Start managing tickets\n\nI think this is kinda working now, yay!	System Admin	published	👋	2025-02-28 06:52:35.090979	2025-03-05 03:06:17.39391	\N	\N	2
3	quick-start	Quick Start Guide	Get up and running quickly with Nosdesk	# Quick Start Guide\n\nThis guide will help you get started with Nosdesk in just a few minutes.\n\n## 1. Create Your Account\n\nIf you haven't already, sign up for a Nosdesk account. You'll need to provide:\n\n* Your name\n\n* Email address\n\n* Company name\n\n* Password\n\n## 2. Set Up Your Team\n\nInvite your team members to join Nosdesk:\n\n1. Go to **Settings > Team**\n\n2. Click **Invite Team Member**\n\n3. Enter their email address and select their role\n\n4. Click **Send Invitation**\n\n## 3. Configure Your Workflow\n\nSet up your ticket categories and statuses:\n\n1. Go to **Settings > Workflow**\n\n2. Create ticket categories that match your support needs\n\n3. Customize ticket statuses to match your process\n\n4. Set up automation rules for ticket routing\n\n## 4. Start Managing Tickets\n\nYou're ready to start using Nosdesk!\n\n1. Create your first ticket\n\n2. Assign it to a team member\n\n3. Track its progress through your workflow\n\n## Need More Help?\n\nCheck out our detailed documentation or contact support if you have any questions.	System Admin	published	⚡	2025-02-28 06:52:35.091458	2025-03-05 02:59:34.723105	2	\N	0
5	creating-tickets	Creating Tickets	Learn how to create and manage support tickets	# Creating Tickets\n\nThis guide explains how to create and manage support tickets in Nosdesk.\n\n## Creating a New Ticket\n\n1. Click the **New Ticket** button in the top navigation bar\n\n2. Fill in the required fields:\n\n   * **Title**: A brief summary of the issue\n\n   * **Description**: Detailed information about the problem\n\n   * **Category**: Select the appropriate category\n\n   * **Priority**: Set the urgency level\n\n3. Add any attachments if needed\n\n4. Click **Create Ticket**\n\n## Assigning Tickets\n\nYou can assign tickets to team members in two ways:\n\n1. During creation by selecting a team member from the **Assign To** dropdown\n\n2. After creation by clicking the **Assign** button on the ticket detail page\n\n## Setting Ticket Priority\n\nNosdesk uses four priority levels:\n\n* **Low**: Minor issues that don't affect workflow\n\n* **Medium**: Issues that cause inconvenience but have workarounds\n\n* **High**: Problems that significantly impact work\n\n* **Critical**: Severe issues that prevent work entirely\n\n## Adding Comments\n\nTo add a comment to a ticket:\n\n1. Open the ticket\n\n2. Scroll to the comment section\n\n3. Type your comment\n\n4. Click **Add Comment**\n\n## Closing Tickets\n\nWhen an issue is resolved:\n\n1. Open the ticket\n\n2. Click the **Close Ticket** button\n\n3. Select a resolution category\n\n4. Add any final comments\n\n5. Click **Confirm**	System Admin	published	📝	2025-02-28 06:52:35.092548	2025-03-08 05:02:42.740159	\N	\N	0
\.


--
-- Data for Name: linked_tickets; Type: TABLE DATA; Schema: public; Owner: kylephillips
--

COPY public.linked_tickets (ticket_id, linked_ticket_id, created_at) FROM stdin;
6	12	2025-03-03 22:35:01.411887
12	6	2025-03-03 22:35:01.411887
47	8	2025-03-04 02:17:01.540163
8	47	2025-03-04 02:17:01.540163
54	8	2025-03-08 05:00:42.709005
8	54	2025-03-08 05:00:42.709005
\.


--
-- Data for Name: project_tickets; Type: TABLE DATA; Schema: public; Owner: kylephillips
--

COPY public.project_tickets (project_id, ticket_id) FROM stdin;
1	44
1	45
1	9
1	1
1	5
1	14
\.


--
-- Data for Name: projects; Type: TABLE DATA; Schema: public; Owner: kylephillips
--

COPY public.projects (id, name, description, status, created_at, updated_at) FROM stdin;
1	Create this helpdesk	Make Nosdesk fully functional	active	2025-03-02 04:47:30.727387	2025-03-02 08:47:41.285721
\.


--
-- Data for Name: tickets; Type: TABLE DATA; Schema: public; Owner: kylephillips
--

COPY public.tickets (id, title, status, priority, created, modified, assignee, requester, closed_at) FROM stdin;
47	Dropped a Chromebook in the Bin	open	medium	2025-03-04 13:16:38.269975	2025-03-04 02:22:07.773	4c105c3b-9bce-4a7c-b93d-97783da1a192	63797ad4-d236-4e8d-a2e9-2aae4a0b0ddf	\N
54	Example Ticket because I can't think of anything better	in-progress	medium	2025-03-08 15:59:10.164946	2025-03-08 16:00:31.814107	4c105c3b-9bce-4a7c-b93d-97783da1a192	63797ad4-d236-4e8d-a2e9-2aae4a0b0ddf	\N
53	Max Cooper - On Being	closed	medium	2025-03-07 13:31:38.95463	2025-03-07 21:32:09.391	\N		2025-03-08 08:32:09.400054
44	Create a mobile app	open	medium	2025-02-28 21:33:07.286966	2025-03-06 02:30:25.543	4c105c3b-9bce-4a7c-b93d-97783da1a192	12345678-1234-1234-1234-123456789012	\N
5	Implement Auth and Login Flow	closed	high	2025-02-27 00:00:00	2025-03-08 05:01:38.231	4c105c3b-9bce-4a7c-b93d-97783da1a192	12345678-1234-1234-1234-123456789012	2025-03-08 16:01:38.250553
14	Update user documentation	in-progress	medium	2025-02-27 00:00:00	2025-03-08 05:01:58.075	4c105c3b-9bce-4a7c-b93d-97783da1a192	12345678-1234-1234-1234-123456789012	\N
12	Implement a decent backend	open	medium	2025-02-27 00:00:00	2025-03-06 01:26:58.933	4c105c3b-9bce-4a7c-b93d-97783da1a192	63797ad4-d236-4e8d-a2e9-2aae4a0b0ddf	\N
10	Create JSON files for client data handling	open	high	2025-02-27 00:00:00	2025-03-02 03:50:27.67	4c105c3b-9bce-4a7c-b93d-97783da1a192	12345678-1234-1234-1234-123456789012	\N
50	Josh's ticket	open	medium	2025-03-06 13:18:56.578284	2025-03-06 03:08:58.54	4c105c3b-9bce-4a7c-b93d-97783da1a192	12345678-1234-1234-1234-123456789012	\N
15	Add more example tickets	open	medium	2025-02-27 00:00:00	2025-03-05 02:44:29.86	4c105c3b-9bce-4a7c-b93d-97783da1a192	12345678-1234-1234-1234-123456789012	\N
9	Write a new custom helpdesk ticketing system	in-progress	medium	2025-02-27 00:00:00	2025-03-06 05:02:44.693	4c105c3b-9bce-4a7c-b93d-97783da1a192	12345678-1234-1234-1234-123456789012	\N
6	Update database backend system	closed	high	2025-02-27 00:00:00	2025-03-02 22:49:26.229	4c105c3b-9bce-4a7c-b93d-97783da1a192	12345678-1234-1234-1234-123456789012	2025-03-02 22:49:26.229
37	Broken Hinge	open	medium	2025-02-28 18:29:03.192004	2025-03-02 03:51:24.211	4c105c3b-9bce-4a7c-b93d-97783da1a192	12345678-1234-1234-1234-123456789012	\N
45	Implement Ticket Notes	closed	high	2025-03-03 10:53:42.261922	2025-03-06 03:10:33.826	4c105c3b-9bce-4a7c-b93d-97783da1a192	4c105c3b-9bce-4a7c-b93d-97783da1a192	2025-03-06 03:10:33.826
43	Implement a ticket time based priority handler	open	medium	2025-02-28 21:32:43.474112	2025-03-02 03:51:29.986	4c105c3b-9bce-4a7c-b93d-97783da1a192	12345678-1234-1234-1234-123456789012	\N
1	The first ticket made with Nosdesk	closed	medium	2025-02-27 00:00:00	2025-03-07 00:50:36.741	4c105c3b-9bce-4a7c-b93d-97783da1a192	12345678-1234-1234-1234-123456789012	2025-03-07 11:50:19.558411
18	Ticket Title Details	open	medium	2025-02-28 00:00:00	2025-03-03 02:36:34.623	4c105c3b-9bce-4a7c-b93d-97783da1a192	12345678-1234-1234-1234-123456789012	\N
8	Boysenberry yoghurt all over laptop	in-progress	medium	2025-02-27 00:00:00	2025-03-02 21:29:36.593	4c105c3b-9bce-4a7c-b93d-97783da1a192	12345678-1234-1234-1234-123456789012	\N
7	Fix search functionality in admin dashboard	open	medium	2025-02-27 00:00:00	2025-03-02 03:49:34.574	4c105c3b-9bce-4a7c-b93d-97783da1a192	12345678-1234-1234-1234-123456789012	\N
\.


--
-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: kylephillips
--

COPY public.users (id, uuid, name, email, role, created_at, updated_at, password_hash) FROM stdin;
2	4c105c3b-9bce-4a7c-b93d-97783da1a192	Kyle Phillips	me@kyle.au	user	2025-03-02 03:46:59.491234	2025-03-02 03:46:59.491234	\\x2432622431322455555837676c2f342f5a794b576562664d533376442e383047365533457377416c4d727473744a626e413962367449744a4e696643
1	12345678-1234-1234-1234-123456789012	Test User	test@example.com	admin	2025-03-02 03:20:49.081947	2025-03-02 03:20:49.081947	\\x24326224313224312f74446b344d42543934467063324f53504851542e4b614753454b795a7171566d53395a696c6266554a6d6d4d494d6b7a353171
3	63797ad4-d236-4e8d-a2e9-2aae4a0b0ddf	Dylan Anderson	dylan.anderson@example.com	user	2025-03-04 02:16:34.288797	2025-03-04 02:16:34.288797	\\x24326224313224725744485a4769546c59585555534c30445a31464d4f2e67787a43746c4e6638486c7241386470676e77706775477258715532502e
4	184ca24d-1520-4828-aacc-8d63476d7eb0	Tony Keen	Tony.Keen@nossalhs.vic.edu.au	user	2025-03-06 03:25:42.848809	2025-03-06 03:25:42.848809	\\x243262243132244d6e586b633369724272677049586f42344d5a54452e74327a5736714d73483736685042303146464d56746c3178534b6e736e7565
\.


--
-- Name: article_contents_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kylephillips
--

SELECT pg_catalog.setval('public.article_contents_id_seq', 42, true);


--
-- Name: attachments_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kylephillips
--

SELECT pg_catalog.setval('public.attachments_id_seq', 26, true);


--
-- Name: comments_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kylephillips
--

SELECT pg_catalog.setval('public.comments_id_seq', 53, true);


--
-- Name: devices_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kylephillips
--

SELECT pg_catalog.setval('public.devices_id_seq', 2, true);


--
-- Name: documentation_pages_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kylephillips
--

SELECT pg_catalog.setval('public.documentation_pages_id_seq', 5, true);


--
-- Name: projects_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kylephillips
--

SELECT pg_catalog.setval('public.projects_id_seq', 2, true);


--
-- Name: tickets_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kylephillips
--

SELECT pg_catalog.setval('public.tickets_id_seq', 54, true);


--
-- Name: users_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kylephillips
--

SELECT pg_catalog.setval('public.users_id_seq', 4, true);


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

