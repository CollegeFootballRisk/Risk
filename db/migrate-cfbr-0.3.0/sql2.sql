update territory_adjacency set max_turn = 999999;
drop view territory_neighbor_history cascade;
CREATE VIEW public.territory_neighbor_history AS
 SELECT territory_ownership.turn_id AS turn_id,
    territory_adjacency.territory_id AS id,
    json_agg(json_build_object('id', territory_ownership.territory_id, 'name', territories.name, 'shortName', territories.name, 'owner', teams.tname)) AS neighbors
   FROM (((public.territory_adjacency
     JOIN public.territory_ownership ON ((territory_ownership.territory_id = territory_adjacency.adjacent_id)))
     JOIN public.teams ON ((teams.id = territory_ownership.owner_id)))
     JOIN public.territories ON ((territories.id = territory_ownership.territory_id)))
  WHERE (territory_adjacency.territory_id <> territory_adjacency.adjacent_id and territory_adjacency.max_turn >= territory_ownership.turn_id and territory_adjacency.min_turn < territory_ownership.turn_id)
  GROUP BY territory_adjacency.territory_id, territory_ownership.turn_id
  ORDER BY territory_adjacency.territory_id;


ALTER TABLE public.territory_neighbor_history OWNER TO risk;

--
-- Name: territory_ownership_with_neighbors; Type: VIEW; Schema: public; Owner: risk
--

CREATE VIEW public.territory_ownership_with_neighbors AS
 SELECT territory_ownership.territory_id,
    turninfo.day,
    turninfo.season,
    territories.name,
    territories.region,
    teams.tname,
    territory_neighbor_history.neighbors
   FROM (((public.territory_ownership
     JOIN public.teams ON ((teams.id = territory_ownership.owner_id)))
     JOIN public.territories ON ((territory_ownership.territory_id = territories.id)))
     JOIN public.turninfo ON ((territory_ownership.turn_id = turninfo.id))
     left JOIN public.territory_neighbor_history ON ((territory_ownership.territory_id = territory_neighbor_history.id and territory_neighbor_history.turn_id = turninfo.id)))
  WHERE ((turninfo.id = territory_neighbor_history.turn_id or territory_neighbor_history.neighbors is null));


ALTER TABLE public.territory_ownership_with_neighbors OWNER TO risk;