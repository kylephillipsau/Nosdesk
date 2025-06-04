#!/bin/bash
set -e

echo "🚀 Starting Nosdesk Backend..."

# Run database migrations
echo "📊 Running database migrations..."
if diesel migration run; then
    echo "✅ Migrations completed successfully"
else
    echo "❌ Migration failed"
    exit 1
fi

# Start the backend application
echo "🎯 Starting backend server..."
exec ./backend 