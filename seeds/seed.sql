--
-- PostgreSQL database dump
--

-- Dumped from database version 11.5
-- Dumped by pg_dump version 11.5

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
-- Name: diesel_manage_updated_at(regclass); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.diesel_manage_updated_at(_tbl regclass) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$;


ALTER FUNCTION public.diesel_manage_updated_at(_tbl regclass) OWNER TO postgres;

--
-- Name: diesel_set_updated_at(); Type: FUNCTION; Schema: public; Owner: postgres
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


ALTER FUNCTION public.diesel_set_updated_at() OWNER TO postgres;

--
-- Name: random_string(integer); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.random_string(length integer) RETURNS text
    LANGUAGE plpgsql
    AS $$
declare
  chars text[] := '{0,1,2,3,4,5,6,7,8,9,A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z}';
  result text := '';
  i integer := 0;
begin
  if length < 0 then
    raise exception 'Given length cannot be less than 0';
  end if;
  for i in 1..length loop
    result := result || chars[1+random()*(array_length(chars, 1)-1)];
  end loop;
  return result;
end;
$$;


ALTER FUNCTION public.random_string(length integer) OWNER TO postgres;

SET default_tablespace = '';

SET default_with_oids = false;

--
-- Name: __diesel_schema_migrations; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.__diesel_schema_migrations (
    version character varying(50) NOT NULL,
    run_on timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public.__diesel_schema_migrations OWNER TO postgres;

--
-- Name: board; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.board (
    id character(16) DEFAULT public.random_string(16) NOT NULL,
    name character varying DEFAULT 'Untitled'::character varying NOT NULL,
    max_votes smallint DEFAULT 1 NOT NULL,
    voting_open boolean DEFAULT false NOT NULL,
    cards_open boolean DEFAULT false NOT NULL
);


ALTER TABLE public.board OWNER TO postgres;

--
-- Name: card; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.card (
    id character(16) DEFAULT public.random_string(16) NOT NULL,
    rank_id character(16) NOT NULL,
    name character varying DEFAULT ''::character varying NOT NULL,
    description character varying DEFAULT ''::character varying NOT NULL
);


ALTER TABLE public.card OWNER TO postgres;

--
-- Name: participant; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.participant (
    id character(16) DEFAULT public.random_string(16) NOT NULL
);


ALTER TABLE public.participant OWNER TO postgres;

--
-- Name: participant_board; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.participant_board (
    participant_id character(16) NOT NULL,
    board_id character(16) NOT NULL,
    owner boolean DEFAULT false NOT NULL
);


ALTER TABLE public.participant_board OWNER TO postgres;

--
-- Name: rank; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.rank (
    id character(16) DEFAULT public.random_string(16) NOT NULL,
    board_id character(16) NOT NULL,
    name character varying DEFAULT ''::character varying NOT NULL
);


ALTER TABLE public.rank OWNER TO postgres;

--
-- Name: vote; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.vote (
    participant_id character(16) NOT NULL,
    card_id character(16) NOT NULL,
    count smallint DEFAULT 1 NOT NULL
);


ALTER TABLE public.vote OWNER TO postgres;

--
-- Data for Name: __diesel_schema_migrations; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.__diesel_schema_migrations (version, run_on) FROM stdin;
00000000000000	2019-10-09 11:56:57.475617
20190707035000	2019-10-09 11:56:57.552647
20190707035013	2019-10-09 11:56:57.625643
20190707035020	2019-10-09 11:56:57.706639
20190707035021	2019-10-09 11:56:57.783682
20190707040022	2019-10-09 11:56:57.862553
20190707040354	2019-10-09 11:56:57.943621
20190707040520	2019-10-09 11:56:58.024666
\.


--
-- Data for Name: board; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.board (id, name, max_votes, voting_open, cards_open) FROM stdin;
jnH28yV52CaE3yPO	Test	1	f	t
\.


--
-- Data for Name: card; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.card (id, rank_id, name, description) FROM stdin;
mLusD4AwVi9v8N7R	K1I8z6S8Wjj3zo0j	Card	There aren't enough snacks :<
hTPUoj2rGiTwLmv8	K1I8z6S8Wjj3zo0j	Card	The backend doesn't expose vote count
8q1KuBHQtRM2nTTW	cIllHHqJTRXWQwuj	Card	You can't pick a rank's color
ws1lc2dsk7o6tlE1	cIllHHqJTRXWQwuj	Card	This meeting is boring
cFMXRdxL5KNsnqPk	cIllHHqJTRXWQwuj	Card	I can't use the heart to represent votes because you're supposed to be able to vote more than once.
kPWMSAFDH4JApXBS	cIllHHqJTRXWQwuj	Card	I need to expand the settings to include all the other things that can be set.
mX0EBxZGIw96oYqY	yErxKK7p4ps4ftnI	Card	Svelte is fun
xMvQWBdnFvx6T9YH	yErxKK7p4ps4ftnI	Card	Rust is also fun
\.


--
-- Data for Name: participant; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.participant (id) FROM stdin;
LLbtnxnL9PGbXG6w
\.


--
-- Data for Name: participant_board; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.participant_board (participant_id, board_id, owner) FROM stdin;
LLbtnxnL9PGbXG6w	jnH28yV52CaE3yPO	t
\.


--
-- Data for Name: rank; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.rank (id, board_id, name) FROM stdin;
K1I8z6S8Wjj3zo0j	jnH28yV52CaE3yPO	Mad
cIllHHqJTRXWQwuj	jnH28yV52CaE3yPO	Sad
yErxKK7p4ps4ftnI	jnH28yV52CaE3yPO	Glad
\.


--
-- Data for Name: vote; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.vote (participant_id, card_id, count) FROM stdin;
\.


--
-- Name: __diesel_schema_migrations __diesel_schema_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.__diesel_schema_migrations
    ADD CONSTRAINT __diesel_schema_migrations_pkey PRIMARY KEY (version);


--
-- Name: board board_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.board
    ADD CONSTRAINT board_pkey PRIMARY KEY (id);


--
-- Name: card card_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.card
    ADD CONSTRAINT card_pkey PRIMARY KEY (id);


--
-- Name: participant_board participant_board_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.participant_board
    ADD CONSTRAINT participant_board_pkey PRIMARY KEY (participant_id, board_id);


--
-- Name: participant participant_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.participant
    ADD CONSTRAINT participant_pkey PRIMARY KEY (id);


--
-- Name: rank rank_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.rank
    ADD CONSTRAINT rank_pkey PRIMARY KEY (id);


--
-- Name: vote vote_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.vote
    ADD CONSTRAINT vote_pkey PRIMARY KEY (card_id, participant_id);


--
-- Name: card card_rank_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.card
    ADD CONSTRAINT card_rank_id_fkey FOREIGN KEY (rank_id) REFERENCES public.rank(id) ON DELETE CASCADE;


--
-- Name: participant_board participant_board_board_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.participant_board
    ADD CONSTRAINT participant_board_board_id_fkey FOREIGN KEY (board_id) REFERENCES public.board(id) ON DELETE CASCADE;


--
-- Name: participant_board participant_board_participant_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.participant_board
    ADD CONSTRAINT participant_board_participant_id_fkey FOREIGN KEY (participant_id) REFERENCES public.participant(id) ON DELETE CASCADE;


--
-- Name: rank rank_board_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.rank
    ADD CONSTRAINT rank_board_id_fkey FOREIGN KEY (board_id) REFERENCES public.board(id) ON DELETE CASCADE;


--
-- Name: vote vote_card_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.vote
    ADD CONSTRAINT vote_card_id_fkey FOREIGN KEY (card_id) REFERENCES public.card(id) ON DELETE CASCADE;


--
-- Name: vote vote_participant_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.vote
    ADD CONSTRAINT vote_participant_id_fkey FOREIGN KEY (participant_id) REFERENCES public.participant(id) ON DELETE CASCADE;


--
-- Name: SCHEMA public; Type: ACL; Schema: -; Owner: cloudsqlsuperuser
--

REVOKE ALL ON SCHEMA public FROM cloudsqladmin;
REVOKE ALL ON SCHEMA public FROM PUBLIC;
GRANT ALL ON SCHEMA public TO cloudsqlsuperuser;
GRANT ALL ON SCHEMA public TO PUBLIC;


--
-- PostgreSQL database dump complete
--

