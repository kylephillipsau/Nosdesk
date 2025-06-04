import axios from 'axios';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || '';

export interface OnboardingStatus {
  requires_setup: boolean;
  user_count: number;
}

export interface AdminSetupRequest {
  name: string;
  email: string;
  password: string;
}

export interface AdminSetupResponse {
  success: boolean;
  message: string;
  user?: any;
}

export interface LoginCredentials {
  email: string;
  password: string;
}

export interface LoginResponse {
  token: string;
  user: any;
}

class AuthService {
  /**
   * Check if the system requires initial setup
   */
  async checkSetupStatus(): Promise<OnboardingStatus> {
    try {
      const response = await axios.get(`${API_BASE_URL}/api/auth/setup/status`);
      return response.data;
    } catch (error) {
      console.error('Error checking setup status:', error);
      throw error;
    }
  }

  /**
   * Setup the initial admin user
   */
  async setupInitialAdmin(adminData: AdminSetupRequest): Promise<AdminSetupResponse> {
    try {
      const response = await axios.post(`${API_BASE_URL}/api/auth/setup/admin`, adminData);
      return response.data;
    } catch (error) {
      console.error('Error setting up initial admin:', error);
      throw error;
    }
  }

  /**
   * Login with email and password
   */
  async login(credentials: LoginCredentials): Promise<LoginResponse> {
    try {
      const response = await axios.post(`${API_BASE_URL}/api/auth/login`, credentials);
      return response.data;
    } catch (error) {
      console.error('Login error:', error);
      throw error;
    }
  }

  /**
   * Get the current authenticated user
   */
  async getCurrentUser(): Promise<any> {
    try {
      const token = localStorage.getItem('authToken');
      if (!token) {
        throw new Error('No auth token found');
      }

      const response = await axios.get(`${API_BASE_URL}/api/auth/me`, {
        headers: {
          'Authorization': `Bearer ${token}`
        }
      });
      return response.data;
    } catch (error) {
      console.error('Error getting current user:', error);
      throw error;
    }
  }

  /**
   * Logout the current user
   */
  logout(): void {
    localStorage.removeItem('authToken');
  }

  /**
   * Check if user is authenticated
   */
  isAuthenticated(): boolean {
    return !!localStorage.getItem('authToken');
  }

  /**
   * Get the stored auth token
   */
  getToken(): string | null {
    return localStorage.getItem('authToken');
  }

  /**
   * Store the auth token
   */
  setToken(token: string): void {
    localStorage.setItem('authToken', token);
  }
}

export const authService = new AuthService();
export default authService; 