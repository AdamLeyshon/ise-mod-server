drop view summary_inventory_votes;
create view summary_inventory_votes as
(
SELECT ni.item_code,
       ni.thing_def,
       ni.quality,
       ni.minified,
       ni.base_value,
       ni.stuff,
       ni.weight,
       ni.version,
       v.votes
FROM new_inventory ni
         LEFT JOIN (
    SELECT nivt.version, count(nivt.version) as votes
    FROM new_inventory_vote_tracker nivt
    GROUP BY nivt.version
) as v on v.version = ni.version
    );

TRUNCATE TABLE public.new_inventory_vote_tracker;
ALTER TABLE ise.public.new_inventory_vote_tracker
    DROP CONSTRAINT new_inventory_vote_tracker_pk;
ALTER TABLE ise.public.new_inventory_vote_tracker
    DROP COLUMN IF EXISTS colony_id;
ALTER TABLE ise.public.new_inventory_vote_tracker
    ADD CONSTRAINT new_inventory_vote_tracker_pk PRIMARY KEY (version, client_bind_id);
