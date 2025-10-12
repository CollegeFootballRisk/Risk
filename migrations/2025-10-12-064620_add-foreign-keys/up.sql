-- Your SQL goes here
ALTER TABLE territories
ADD CONSTRAINT fk_region FOREIGN KEY (region) REFERENCES regions(id);