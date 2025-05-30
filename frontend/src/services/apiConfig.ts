import axios from 'axios';

// Set API URL based on environment
export const API_URL = import.meta.env.VITE_API_URL || '/api';

// Create axios instance with default config
const apiClient = axios.create({
  baseURL: API_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Add request interceptor for authentication
apiClient.interceptors.request.use(
  (config) => {
    // Get token from localStorage
    const token = localStorage.getItem('token');
    const authProvider = localStorage.getItem('authProvider');
    
    // If token exists, add it to the headers
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
      
      // Add auth provider header if available
      if (authProvider) {
        config.headers['X-Auth-Provider'] = authProvider;
      }
    }
    
    // Enable debugging in development
    if (import.meta.env.DEV) {
      console.log(`API Request to ${config.url}:`, { 
        method: config.method,
        headers: config.headers,
        data: config.data,
      });
    }
    
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// Add response interceptor for error handling
apiClient.interceptors.response.use(
  (response) => {
    // Enable debugging in development
    if (import.meta.env.DEV) {
      // Get the call stack to identify which component made the request
      const stack = new Error().stack;
      const caller = stack?.split('\n')[4]?.trim() || 'Unknown caller';
      
      console.log(`API Response from ${response.config.url}:`, { 
        status: response.status,
        data: response.data,
        caller: caller
      });
    }
    
    return response;
  },
  (error) => {
    // Handle authentication errors
    if (error.response && error.response.status === 401) {
      // If we're already on the login page, don't redirect
      if (!window.location.pathname.includes('/login')) {
        // Clear user token and redirect to login
        localStorage.removeItem('token');
        localStorage.removeItem('authProvider');
        window.location.href = '/login';
      }
    }
    
    // Log error details in development
    if (import.meta.env.DEV) {
      console.error('API Error:', {
        message: error.message,
        config: error.config,
        response: error.response ? {
          status: error.response.status,
          data: error.response.data
        } : 'No response'
      });
    }
    
    return Promise.reject(error);
  }
);

export default apiClient; 