-- Create Subscriptions Table
CREATE TABLE subscriptions(
    id uuid PRIMARY KEY,
    email text NOT NULL UNIQUE,
    name text NOT NULL,
    subscribed_at timestamptz NOT NULL
);