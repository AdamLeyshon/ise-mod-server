DROP VIEW IF EXISTS summary_inventory_votes;

TRUNCATE TABLE new_inventory_vote_tracker;
ALTER TABLE new_inventory_vote_tracker
    ADD COLUMN IF NOT EXISTS colony_id uuid NOT NULL;
ALTER TABLE new_inventory_vote_tracker
    DROP CONSTRAINT IF EXISTS new_inventory_vote_tracker_pk;
ALTER TABLE new_inventory_vote_tracker
    ADD CONSTRAINT new_inventory_vote_tracker_pk PRIMARY KEY (version, client_bind_id, colony_id);

CREATE VIEW summary_inventory_votes AS (
SELECT
   ni.item_code,
   ni.thing_def,
   ni.quality,
   ni.minified,
   ni.base_value,
   ni.stuff,
   ni.weight,
   ni.version,
   v.votes
FROM
   new_inventory ni
       LEFT JOIN (
       SELECT
           version,
           COUNT(version) AS votes
       FROM
           (
               SELECT
                   DISTINCT ON (version, client_bind_id) version
               FROM
                   new_inventory_vote_tracker
           ) AS vt
       GROUP BY
           version
   ) AS v ON v.version = ni.version
WHERE
       v.votes > 0
   );

