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
         LEFT JOIN (SELECT nivt.version,
                           count(nivt.version) AS votes
                    FROM new_inventory_vote_tracker nivt
                    GROUP BY nivt.version) v ON v.version::text = ni.version::text
         LEFT JOIN inventory i ON ni.version::text = i.version::text
WHERE i.version IS NULL
    );
