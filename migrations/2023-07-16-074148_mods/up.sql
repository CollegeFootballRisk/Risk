-- Your SQL goes here
insert into player_role (role_id, player_id)
select
    (select id from role where name = 'Moderator'),
    "player".id
from "player"
where
    "player".id in (select player_id from authentication_method where platform = 'reddit' and foreign_id in 
    ('Mautamu', 'SnareShot', 'lAMA_Bear_AMA', 'narcolepszzz', 'jakers0516', 'Girdon_Freeman', 'bakonydraco', 'Belgara', 'EpicWolverine', 'GoCardinal07', 'igloo27', 'invertthatveer', 'littlemojo', 'Inspector_Tortoise', 'semicorrect', '-MrWrightt-', 'Sup3rtom2000')
  );

insert into player_role (role_id, player_id)
select
    (select id from role where name = 'Security'),
    "player".id
from "player"
where
    "player".id in (select player_id from authentication_method where platform = 'reddit' and foreign_id in 
    ('Mautamu')
  );
