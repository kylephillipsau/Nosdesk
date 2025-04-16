// API connection
export const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api';

// Default timeout for API requests
export const DEFAULT_TIMEOUT = 30000; // 30 seconds

// Collaboration settings
export const MAX_SYNC_RETRY = 3;
export const SYNC_INTERVAL = 2000; // 2 seconds 