-- Your SQL goes here
ALTER TABLE territory_adjacency
ADD CONSTRAINT fk_territory_adjacency_territory_id FOREIGN KEY (territory_id) REFERENCES territories(id) ON DELETE CASCADE;
ALTER TABLE territory_adjacency
ADD CONSTRAINT fk_territory_adjacency_adjacent_id FOREIGN KEY (adjacent_id) REFERENCES territories(id) ON DELETE CASCADE;
ALTER TABLE stats
ADD CONSTRAINT fk_stats_team FOREIGN KEY (team) REFERENCES teams(id) ON DELETE CASCADE;
ALTER TABLE stats
ADD CONSTRAINT fk_stats_turn_id FOREIGN KEY (turn_id) REFERENCES turninfo(id) ON DELETE CASCADE;
ALTER TABLE territory_stats
ADD CONSTRAINT fk_territory_stats_team FOREIGN KEY (team) REFERENCES teams(id) ON DELETE CASCADE;
ALTER TABLE territory_stats
ADD CONSTRAINT fk_territory_stats_turn_id FOREIGN KEY (turn_id) REFERENCES turninfo(id) ON DELETE CASCADE;
ALTER TABLE users
ADD CONSTRAINT fk_users_current_team FOREIGN KEY (current_team) REFERENCES teams(id) ON DELETE
SET NULL;
ALTER TABLE users
ADD CONSTRAINT fk_users_playing_for FOREIGN KEY (playing_for) REFERENCES teams(id) ON DELETE
SET NULL;
ALTER TABLE territory_ownership
ADD CONSTRAINT fk_territory_ownership_territory_id FOREIGN KEY (territory_id) REFERENCES territories(id) ON DELETE CASCADE;
ALTER TABLE territory_ownership
ADD CONSTRAINT fk_territory_ownership_owner_id FOREIGN KEY (owner_id) REFERENCES teams(id) ON DELETE CASCADE;
ALTER TABLE territory_ownership
ADD CONSTRAINT fk_territory_ownership_previous_owner_id FOREIGN KEY (previous_owner_id) REFERENCES teams(id) ON DELETE
SET NULL;
ALTER TABLE territory_ownership
ADD CONSTRAINT fk_territory_ownership_mvp FOREIGN KEY (mvp) REFERENCES users(id) ON DELETE
SET NULL;
ALTER TABLE territory_ownership
ADD CONSTRAINT fk_territory_ownership_turn_id FOREIGN KEY (turn_id) REFERENCES turninfo(id) ON DELETE CASCADE;
ALTER TABLE territories
ADD CONSTRAINT fk_territories_region FOREIGN KEY (region) REFERENCES regions(id) ON DELETE
SET NULL;
ALTER TABLE turns
ADD CONSTRAINT fk_turns_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE turns
ADD CONSTRAINT fk_turns_territory FOREIGN KEY (territory) REFERENCES territories(id) ON DELETE CASCADE;
ALTER TABLE turns
ADD CONSTRAINT fk_turns_turn_id FOREIGN KEY (turn_id) REFERENCES turninfo(id) ON DELETE CASCADE;
ALTER TABLE continuation_responses
ADD CONSTRAINT fk_continuation_responses_poll_id FOREIGN KEY (poll_id) REFERENCES continuation_polls(id) ON DELETE CASCADE;
ALTER TABLE continuation_responses
ADD CONSTRAINT fk_continuation_responses_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE continuation_polls
ADD CONSTRAINT fk_continuation_polls_turn_id FOREIGN KEY (turn_id) REFERENCES turninfo(id) ON DELETE CASCADE;
ALTER TABLE audit_log
ADD CONSTRAINT fk_audit_log_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;