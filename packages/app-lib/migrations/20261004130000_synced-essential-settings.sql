INSERT INTO sync_feature_settings
	(feature, globally_enabled, new_instance_default)
VALUES ('essential_settings', 0, 1)
ON CONFLICT(feature) DO NOTHING;

INSERT INTO instance_sync_preferences (instance_id, feature, enabled)
SELECT instances.id, 'essential_settings', 1
FROM instances
WHERE true
ON CONFLICT(instance_id, feature) DO NOTHING;
