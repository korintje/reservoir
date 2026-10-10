DROP TRIGGER IF EXISTS reservation_insert_rules;
DROP TRIGGER IF EXISTS reservation_update_rules;
DROP TRIGGER IF EXISTS reservation_delete_rules;
DROP TRIGGER IF EXISTS user_preserve_history;
DROP TRIGGER IF EXISTS resource_preserve_history;
CREATE TABLE IF NOT EXISTS reservation_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    advance_days INTEGER NOT NULL CHECK (advance_days BETWEEN 1 AND 3650)
);
INSERT OR IGNORE INTO reservation_settings VALUES (1, 15);
CREATE INDEX IF NOT EXISTS reservations_resource_time ON reservations(resource_id, start, end);

CREATE TRIGGER IF NOT EXISTS reservation_insert_rules BEFORE INSERT ON reservations BEGIN
    SELECT CASE WHEN NEW.start >= NEW.end OR NEW.start < -62135596800 OR NEW.end > 253402300799 THEN RAISE(ABORT, 'invalid_interval') END;
    SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM users WHERE id = NEW.user_id AND active = 1)
        THEN RAISE(ABORT, 'inactive_user') END;
    SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM resources WHERE id = NEW.resource_id AND active = 1)
        THEN RAISE(ABORT, 'inactive_resource') END;
    SELECT CASE WHEN NEW.end > CAST(strftime('%s','now') AS INTEGER) +
        (SELECT advance_days * 86400 FROM reservation_settings WHERE id = 1)
        THEN RAISE(ABORT, 'advance_limit') END;
    SELECT CASE WHEN EXISTS (SELECT 1 FROM reservations WHERE resource_id = NEW.resource_id
        AND start < NEW.end AND end > NEW.start) THEN RAISE(ABORT, 'reservation_overlap') END;
END;

CREATE TRIGGER IF NOT EXISTS reservation_update_rules BEFORE UPDATE ON reservations BEGIN
    SELECT CASE WHEN OLD.end <= CAST(strftime('%s','now') AS INTEGER)
        THEN RAISE(ABORT, 'finished_reservation') END;
    SELECT CASE WHEN OLD.start <= CAST(strftime('%s','now') AS INTEGER) AND NEW.start != OLD.start
        THEN RAISE(ABORT, 'in_use_reservation') END;
    SELECT CASE WHEN NEW.start >= NEW.end OR NEW.start < -62135596800 OR NEW.end > 253402300799 THEN RAISE(ABORT, 'invalid_interval') END;
    SELECT CASE WHEN (NEW.user_id != OLD.user_id OR NEW.resource_id != OLD.resource_id)
        AND NOT EXISTS (SELECT 1 FROM users WHERE id = NEW.user_id AND active = 1)
        THEN RAISE(ABORT, 'inactive_user') END;
    SELECT CASE WHEN (NEW.user_id != OLD.user_id OR NEW.resource_id != OLD.resource_id)
        AND NOT EXISTS (SELECT 1 FROM resources WHERE id = NEW.resource_id AND active = 1)
        THEN RAISE(ABORT, 'inactive_resource') END;
    SELECT CASE WHEN (NEW.start != OLD.start OR NEW.end != OLD.end) AND
        NEW.end > CAST(strftime('%s','now') AS INTEGER) +
        (SELECT advance_days * 86400 FROM reservation_settings WHERE id = 1)
        THEN RAISE(ABORT, 'advance_limit') END;
    SELECT CASE WHEN EXISTS (SELECT 1 FROM reservations WHERE id != OLD.id AND resource_id = NEW.resource_id
        AND start < NEW.end AND end > NEW.start) THEN RAISE(ABORT, 'reservation_overlap') END;
END;

CREATE TRIGGER IF NOT EXISTS reservation_delete_rules BEFORE DELETE ON reservations BEGIN
    SELECT CASE WHEN OLD.end <= CAST(strftime('%s','now') AS INTEGER)
        THEN RAISE(ABORT, 'finished_reservation') END;
    SELECT CASE WHEN OLD.start <= CAST(strftime('%s','now') AS INTEGER)
        THEN RAISE(ABORT, 'in_use_reservation') END;
END;
CREATE TRIGGER IF NOT EXISTS user_preserve_history BEFORE DELETE ON users
WHEN EXISTS (SELECT 1 FROM reservations WHERE user_id = OLD.id) BEGIN
    SELECT RAISE(ABORT, 'referenced_user');
END;
CREATE TRIGGER IF NOT EXISTS resource_preserve_history BEFORE DELETE ON resources
WHEN EXISTS (SELECT 1 FROM reservations WHERE resource_id = OLD.id) BEGIN
    SELECT RAISE(ABORT, 'referenced_resource');
END;


CREATE TRIGGER reservation_version AFTER UPDATE ON reservations WHEN NEW.version = OLD.version BEGIN
 UPDATE reservations SET version=OLD.version+1 WHERE id=NEW.id;
END;
