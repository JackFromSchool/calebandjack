CREATE TABLE sessions (
   session_id serial primary key,
   user_id integer not null,
   issued timestamp with time zone default current_timestamp
);
