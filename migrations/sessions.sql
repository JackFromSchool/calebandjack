CREATE TABLE sessions (
   session_id serial primary key,
   user_id integer not null,
   issued timestamp default current_timestamp
);
