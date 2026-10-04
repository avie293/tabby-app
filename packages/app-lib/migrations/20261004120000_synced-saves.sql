INSERT INTO sync_feature_settings
	(feature, globally_enabled, new_instance_default)
VALUES ('saves', 0, 0)
ON CONFLICT(feature) DO NOTHING;
