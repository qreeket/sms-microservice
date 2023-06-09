-- create sms_verification table with props: id, created_at, phone_number
CREATE TABLE sms_verification (
  id SERIAL PRIMARY KEY,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  phone_number VARCHAR(255) NOT NULL
);

-- create a procedure to insert a new user
CREATE OR REPLACE FUNCTION insert_user(user_phone_number VARCHAR(255))
RETURNS VOID AS $$
BEGIN
  INSERT INTO sms_verification (phone_number) VALUES (user_phone_number);
END;
$$ LANGUAGE plpgsql;

-- create a procedure to get a user by phone number
CREATE OR REPLACE FUNCTION get_user_by_phone_number(user_phone_number VARCHAR(255))
RETURNS TABLE(id INTEGER, created_at TIMESTAMP, phone_number VARCHAR(255)) AS $$
BEGIN
  RETURN QUERY SELECT * FROM sms_verification WHERE phone_number = user_phone_number;
END;
$$ LANGUAGE plpgsql;

-- create a procedure to delete a user by phone number
CREATE OR REPLACE FUNCTION delete_user_by_phone_number(user_phone_number VARCHAR(255))
RETURNS VOID AS $$
BEGIN
  DELETE FROM sms_verification WHERE phone_number = user_phone_number;
END;
$$ LANGUAGE plpgsql;

-- create a procedure to check if the user created_at is older than 10 minutes
CREATE OR REPLACE FUNCTION is_user_created_at_older_than_10_minutes(user_phone_number VARCHAR(255))
RETURNS BOOLEAN AS $$
DECLARE
  user_created_at TIMESTAMP;
BEGIN
    SELECT created_at INTO user_created_at FROM sms_verification WHERE phone_number = user_phone_number;
    RETURN user_created_at < NOW() - INTERVAL '10 minutes';
    END;
$$ LANGUAGE plpgsql;
