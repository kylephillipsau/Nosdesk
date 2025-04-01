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

  // Load user from localStorage on initialization
  try {
    const storedUser = localStorage.getItem('user');
    if (storedUser && storedUser !== "undefined" && storedUser !== "null") {
      user.value = JSON.parse(storedUser);
    } else {
      // Clear invalid storage
      localStorage.removeItem('user');
    }
  } catch (e) {
    console.error('Error parsing stored user:', e);
    localStorage.removeItem('user');
  }

  // Computed properties
  const isAuthenticated = computed(() => !!token.value);
  const isAdmin = computed(() => user.value?.role === 'admin');
  const isTechnician = computed(() => user.value?.role === 'technician' || user.value?.role === 'admin');

  // Actions
  async function login(credentials: LoginCredentials) {
    loading.value = true;
    error.value = null;
    
    try {
      const response = await axios.post('/api/auth/login', credentials);
      
      // Store token and user in localStorage
      token.value = response.data.token;
      user.value = response.data.user;
      
      if (token.value) {
        localStorage.setItem('token', token.value);
        // Set Authorization header for future requests
        axios.defaults.headers.common['Authorization'] = `Bearer ${token.value}`;
      }
      
      if (user.value) {
        localStorage.setItem('user', JSON.stringify(user.value));
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

  function logout() {
    // Clear token and user
    token.value = null;
    user.value = null;
    
    // Remove from localStorage
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    
    // Remove Authorization header
    delete axios.defaults.headers.common['Authorization'];
    
    // Redirect to login page
    router.push('/login');
  }

  // Initialize axios with token if it exists
  if (token.value) {
    axios.defaults.headers.common['Authorization'] = `Bearer ${token.value}`;
  }

  return {
    token,
    user,
    loading,
    error,
    isAuthenticated,
    isAdmin,
    isTechnician,
    login,
    logout
  };
}); 