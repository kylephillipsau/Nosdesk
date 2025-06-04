import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import axios from 'axios';
import router from '@/router';

// Configure axios to use relative URLs
// This will make requests go to the same server that served the frontend
axios.defaults.baseURL = '';

interface User {
  id: number;
  uuid: string;
  name: string;
  email: string;
  role: string;
  pronouns?: string | null;
  avatar_url?: string | null;
  banner_url?: string | null;
  avatar_thumb?: string | null;
  created_at: string;
  updated_at: string;
}

interface LoginCredentials {
  email: string;
  password: string;
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem('token'));
  const user = ref<User | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const authProvider = ref<string | null>(localStorage.getItem('authProvider'));

  // Set up axios auth header if token exists
  if (token.value) {
    axios.defaults.headers.common['Authorization'] = `Bearer ${token.value}`;
    
    // Set auth provider header if available
    if (authProvider.value) {
      axios.defaults.headers.common['X-Auth-Provider'] = authProvider.value;
    }
    
    // Load user data from backend when store initializes with a token
    fetchUserData().catch(err => {
      console.error('Failed to load initial user data:', err);
    });
  }

  // Computed properties
  const isAuthenticated = computed(() => !!token.value);
  const isAdmin = computed(() => user.value?.role === 'admin');
  const isTechnician = computed(() => user.value?.role === 'technician' || user.value?.role === 'admin');
  const isMicrosoftAuth = computed(() => authProvider.value === 'microsoft');

  // Helper to detect token type
  function detectTokenType(tokenStr: string): string {
    // Microsoft Entra tokens typically have distinctive characteristics
    if (tokenStr.length > 500 && tokenStr.includes('eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1')) {
      return 'microsoft';
    }
    return 'local';
  }

  // Fetch current user data from the backend
  async function fetchUserData() {
    if (!token.value) return null;

    try {
      loading.value = true;
      // Only log in development or when explicitly requested
      if (import.meta.env.DEV) {
        console.log('Fetching user data...');
      }
      
      const headers: Record<string, string> = {
        'Authorization': `Bearer ${token.value}`
      };
      
      // Add provider header if we have it or can detect it
      if (authProvider.value) {
        headers['X-Auth-Provider'] = authProvider.value;
      } else if (token.value) {
        // Try to detect token type
        const detectedProvider = detectTokenType(token.value);
        if (detectedProvider === 'microsoft') {
          headers['X-Auth-Provider'] = 'microsoft';
          authProvider.value = 'microsoft';
          localStorage.setItem('authProvider', 'microsoft');
        }
      }
      
      const response = await axios.get('/api/auth/me', { headers });
      user.value = response.data;
      return response.data;
    } catch (err: any) {
      console.error('Error fetching user data:', err);
      
      // Only logout on unauthorized errors, not on network/server errors
      if (err.response && (err.response.status === 401 || err.response.status === 403)) {
        console.log('Logging out due to authentication error:', err.response.status);
        // Token is definitely invalid or expired
        logout();
      } else {
        // For other errors (network, server, etc.), just throw the error
        // but keep the user logged in
        error.value = 'Failed to load profile data. Please try again.';
        throw err;
      }
      
      return null;
    } finally {
      loading.value = false;
    }
  }

  // Actions
  async function login(credentials: LoginCredentials) {
    loading.value = true;
    error.value = null;
    
    try {
      const response = await axios.post('/api/auth/login', credentials);
      
      // Store only token in localStorage, not user data
      token.value = response.data.token;
      user.value = response.data.user;
      
      if (token.value) {
        localStorage.setItem('token', token.value);
        // Set provider to local for regular login
        authProvider.value = 'local';
        localStorage.setItem('authProvider', 'local');
        // Set Authorization header for future requests
        axios.defaults.headers.common['Authorization'] = `Bearer ${token.value}`;
        axios.defaults.headers.common['X-Auth-Provider'] = 'local';
      }
      
      // Redirect to dashboard
      router.push('/');
      
      return true;
    } catch (err: any) {
      console.error('Login error:', err);
      error.value = err.response?.data?.message || 'Login failed. Please check your credentials.';
      return false;
    } finally {
      loading.value = false;
    }
  }

  // Handle external auth (Microsoft, etc.)
  async function setExternalAuth(tokenStr: string, userData: User | null, provider: string = 'microsoft') {
    token.value = tokenStr;
    user.value = userData;
    authProvider.value = provider;
    
    if (token.value) {
      localStorage.setItem('token', token.value);
      localStorage.setItem('authProvider', provider);
      axios.defaults.headers.common['Authorization'] = `Bearer ${token.value}`;
      axios.defaults.headers.common['X-Auth-Provider'] = provider;
      
      // If no user data was provided, fetch it from the backend
      if (!userData) {
        try {
          await fetchUserData();
        } catch (err) {
          console.error('Failed to fetch user data after external auth:', err);
          // Don't throw error here - authentication was successful
        }
      }
    }
    
    return true;
  }

  function logout() {
    // Clear token and user
    token.value = null;
    user.value = null;
    authProvider.value = null;
    
    // Remove from localStorage
    localStorage.removeItem('token');
    localStorage.removeItem('authProvider');
    
    // Remove Authorization header
    delete axios.defaults.headers.common['Authorization'];
    delete axios.defaults.headers.common['X-Auth-Provider'];
    
    // Redirect to login page
    router.push('/login');
  }

  return {
    token,
    user,
    loading,
    error,
    authProvider,
    isAuthenticated,
    isAdmin,
    isTechnician,
    isMicrosoftAuth,
    login,
    logout,
    fetchUserData,
    setExternalAuth
  };
}); 