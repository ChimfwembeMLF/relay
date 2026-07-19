ALTER TABLE webhook_delivery_attempts
    ADD COLUMN IF NOT EXISTS event_type TEXT NOT NULL DEFAULT 'payment.status_changed';
