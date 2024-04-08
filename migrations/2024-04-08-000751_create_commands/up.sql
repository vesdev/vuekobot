CREATE TABLE ttv_commands (
    id UUID primary KEY,
    channel VARCHAR NOT NULL,
    command VARCHAR NOT NULL,
    value TEXT NOT NULL
)
