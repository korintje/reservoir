UPDATE app_settings SET notification_text = 'Notify the resource Slack channel manually when delaying, shortening, or cancelling a booking starting within 24 hours, or shortening a booking in progress.'
WHERE notification_text = '予約を短縮・取消した場合は、設備のSlackチャネルへ連絡してください。';
