import apiClient from './apiConfig';

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

// MFA Interfaces
export interface MFASetupData {
  secret: string;
  qr_code: string;
  backup_codes: string[];
}

export interface MFAStatusResponse {
  enabled: boolean;
  has_backup_codes?: boolean;
}

export interface MFAVerifyRequest {
  token: string;
  secret: string;
}

export interface MFAEnableRequest {
  token: string;
  secret: string;
  password?: string;
}

export interface MFALoginSetupRequest {
  email: string;
  password: string;
}

export interface MFALoginEnableRequest {
  email: string;
  password: string;
  token: string;
  secret: string;
  backup_codes: string[];
}

class AuthService {
  private setupStatusCache: {
    data: OnboardingStatus | null;
    timestamp: number;
    ttl: number; // time to live in milliseconds
  } = {
    data: null,
    timestamp: 0,
    ttl: 5 * 60 * 1000 // Cache for 5 minutes
  };
  
  // Security: Prevent multiple simultaneous setup checks
  private setupCheckInProgress: boolean = false;
  private setupCheckPromise: Promise<OnboardingStatus> | null = null;

  /**
   * Check if the system requires initial setup
   * Security: Prevents multiple simultaneous checks and includes rate limiting
   */
  async checkSetupStatus(): Promise<OnboardingStatus> {
    // Security: Check if we have cached data that's still valid
    const now = Date.now();
    if (this.setupStatusCache.data && 
        (now - this.setupStatusCache.timestamp) < this.setupStatusCache.ttl) {
      return this.setupStatusCache.data;
    }

    // Security: Prevent multiple simultaneous setup checks
    if (this.setupCheckInProgress && this.setupCheckPromise) {
      console.log('🔄 AuthService: Setup check already in progress, waiting...');
      return this.setupCheckPromise;
    }

    // Security: Rate limiting - prevent excessive calls
    const timeSinceLastCheck = now - this.setupStatusCache.timestamp;
    if (timeSinceLastCheck < 1000) { // Minimum 1 second between checks
      console.warn('⚠️  AuthService: Rate limiting setup status checks');
      if (this.setupStatusCache.data) {
        return this.setupStatusCache.data;
      }
      // Wait for the minimum interval
      await new Promise(resolve => setTimeout(resolve, 1000 - timeSinceLastCheck));
    }

    // Set up the promise for this check
    this.setupCheckInProgress = true;
    this.setupCheckPromise = this._performSetupStatusCheck();
    
    try {
      const result = await this.setupCheckPromise;
      return result;
    } finally {
      this.setupCheckInProgress = false;
      this.setupCheckPromise = null;
    }
  }

  /**
   * Private method to perform the actual setup status check
   * Security: Separated to prevent race conditions
   */
  private async _performSetupStatusCheck(): Promise<OnboardingStatus> {
    try {
      const response = await apiClient.get(`/auth/setup/status`, {
        timeout: 10000, // 10 second timeout for security
      });
      
      // Security: Validate response data
      if (!response.data || typeof response.data.requires_setup !== 'boolean') {
        console.error('Invalid setup status response format');
        throw new Error('Invalid setup status response format');
      }
      
      // Cache the response
      this.setupStatusCache.data = response.data;
      this.setupStatusCache.timestamp = Date.now();
      
      console.log('🔄 AuthService: Setup status checked:', response.data);
      return response.data;
    } catch (error: any) {
      console.error('Error checking setup status:', error);
      
      // Security: If it's a network error or server error, assume setup is required
      // This ensures new users can still access onboarding even if there are temporary issues
      if (error.code === 'NETWORK_ERROR' || error.code === 'ERR_NETWORK' || 
          (error.response && error.response.status >= 500)) {
        console.warn('Network/server error, assuming setup required for safety');
        return {
          requires_setup: true,
          user_count: 0
        };
      }
      
      throw error;
    }
  }

  /**
   * Setup the initial admin user
   */
  async setupInitialAdmin(adminData: AdminSetupRequest): Promise<AdminSetupResponse> {
    try {
      const response = await apiClient.post('/auth/setup/admin', adminData);
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
      const response = await apiClient.post('/auth/login', credentials);
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
      const response = await apiClient.get('/auth/me');
      return response.data;
    } catch (error) {
      console.error('Error getting current user:', error);
      throw error;
    }
  }

  /**
   * Logout the current user (clears httpOnly cookies on server)
   */
  async logout(): Promise<void> {
    try {
      await apiClient.post('/auth/logout');
    } catch (error) {
      console.error('Error logging out:', error);
      throw error;
    }
  }

  /**
   * Check if user is authenticated (by checking for CSRF token cookie)
   */
  isAuthenticated(): boolean {
    return !!document.cookie.match(/csrf_token=([^;]+)/);
  }

  /**
   * Clear the setup status cache (useful after admin setup completes)
   */
  clearSetupStatusCache(): void {
    this.setupStatusCache.data = null;
    this.setupStatusCache.timestamp = 0;
    console.log('🔄 AuthService: Setup status cache cleared');
  }

  /**
   * Change the user's password
   */
  async changePassword(currentPassword: string, newPassword: string): Promise<void> {
    try {
      await apiClient.post('/auth/change-password', {
        current_password: currentPassword,
        new_password: newPassword
      });
    } catch (error) {
      console.error('Error changing password:', error);
      throw error;
    }
  }

  /**
   * Get all active sessions for the current user
   */
  async getSessions(): Promise<any[]> {
    try {
      const response = await apiClient.get('/auth/sessions');
      return response.data.sessions;
    } catch (error) {
      console.error('Error getting sessions:', error);
      throw error;
    }
  }

  /**
   * Revoke a specific session
   */
  async revokeSession(sessionId: number): Promise<void> {
    try {
      await apiClient.delete(`/auth/sessions/${sessionId}`);
    } catch (error) {
      console.error('Error revoking session:', error);
      throw error;
    }
  }

  /**
   * Revoke all other sessions (keep current session)
   */
  async revokeAllOtherSessions(): Promise<void> {
    try {
      await apiClient.delete('/auth/sessions/others');
    } catch (error) {
      console.error('Error revoking all other sessions:', error);
      throw error;
    }
  }

  // ===== MFA METHODS =====

  /**
   * Setup MFA for login (unauthenticated - used during login flow)
   */
  async setupMFAForLogin(request: MFALoginSetupRequest): Promise<MFASetupData> {
    try {
      const response = await apiClient.post('/auth/mfa-setup-login', request);
      return response.data;
    } catch (error) {
      console.error('Error setting up MFA for login:', error);
      throw error;
    }
  }

  /**
   * Enable MFA during login flow
   */
  async enableMFAForLogin(request: MFALoginEnableRequest): Promise<LoginResponse> {
    try {
      const response = await apiClient.post('/auth/mfa-enable-login', request);
      return response.data;
    } catch (error) {
      console.error('Error enabling MFA for login:', error);
      throw error;
    }
  }

  /**
   * Setup MFA for authenticated user
   */
  async setupMFA(): Promise<MFASetupData> {
    try {
      const response = await apiClient.post('/auth/mfa/setup');
      return response.data;
    } catch (error) {
      console.error('Error setting up MFA:', error);
      throw error;
    }
  }

  /**
   * Verify MFA token during setup
   */
  async verifyMFA(request: MFAVerifyRequest): Promise<{ valid: boolean }> {
    try {
      const response = await apiClient.post('/auth/mfa/verify', request);
      return response.data;
    } catch (error) {
      console.error('Error verifying MFA:', error);
      throw error;
    }
  }

  /**
   * Enable MFA for authenticated user
   */
  async enableMFA(request: MFAEnableRequest): Promise<{ success: boolean; backup_codes: string[] }> {
    try {
      const response = await apiClient.post('/auth/mfa/enable', request);
      return response.data;
    } catch (error) {
      console.error('Error enabling MFA:', error);
      throw error;
    }
  }

  /**
   * Disable MFA
   */
  async disableMFA(password: string): Promise<{ success: boolean }> {
    try {
      const response = await apiClient.post('/auth/mfa/disable', { password });
      return response.data;
    } catch (error) {
      console.error('Error disabling MFA:', error);
      throw error;
    }
  }

  /**
   * Get MFA status
   */
  async getMFAStatus(): Promise<MFAStatusResponse> {
    try {
      const response = await apiClient.get('/auth/mfa/status');
      return response.data;
    } catch (error) {
      console.error('Error getting MFA status:', error);
      throw error;
    }
  }

  /**
   * Regenerate backup codes
   */
  async regenerateBackupCodes(password: string): Promise<{ backup_codes: string[] }> {
    try {
      const response = await apiClient.post('/auth/mfa/regenerate-backup-codes', { password });
      return response.data;
    } catch (error) {
      console.error('Error regenerating backup codes:', error);
      throw error;
    }
  }

  /**
   * Login with MFA token
   */
  async loginWithMFA(email: string, password: string, mfaToken: string, userUuid: string): Promise<LoginResponse> {
    try {
      const response = await apiClient.post('/auth/mfa-login', {
        email,
        password,
        mfa_token: mfaToken,
        user_uuid: userUuid
      });
      return response.data;
    } catch (error) {
      console.error('Error logging in with MFA:', error);
      throw error;
    }
  }

  /**
   * Get OAuth auth providers
   */
  async getAuthProviders(): Promise<any[]> {
    try {
      const response = await apiClient.get('/auth/providers');
      return response.data;
    } catch (error) {
      console.error('Error getting auth providers:', error);
      throw error;
    }
  }

  /**
   * Connect OAuth provider
   */
  async connectOAuthProvider(providerType: string): Promise<{ auth_url?: string }> {
    try {
      const response = await apiClient.post('/auth/oauth/connect', {
        provider_type: providerType
      });
      return response.data;
    } catch (error) {
      console.error('Error connecting OAuth provider:', error);
      throw error;
    }
  }

  /**
   * Delete OAuth provider
   */
  async deleteAuthProvider(providerId: string): Promise<void> {
    try {
      await apiClient.delete(`/auth/providers/${providerId}`);
    } catch (error) {
      console.error('Error deleting auth provider:', error);
      throw error;
    }
  }

  /**
   * Get user's connected auth identities
   */
  async getUserAuthIdentities(): Promise<any[]> {
    try {
      const response = await apiClient.get('/users/auth-identities');
      return response.data;
    } catch (error) {
      console.error('Error getting user auth identities:', error);
      throw error;
    }
  }

  /**
   * Delete user's auth identity
   */
  async deleteUserAuthIdentity(identityId: number): Promise<void> {
    try {
      await apiClient.delete(`/users/auth-identities/${identityId}`);
    } catch (error) {
      console.error('Error deleting user auth identity:', error);
      throw error;
    }
  }

  /**
   * Request password reset
   */
  async requestPasswordReset(email: string): Promise<{ message: string }> {
    try {
      const response = await apiClient.post('/auth/password-reset/request', { email });
      return response.data;
    } catch (error) {
      console.error('Error requesting password reset:', error);
      throw error;
    }
  }

  /**
   * Complete password reset
   */
  async completePasswordReset(token: string, newPassword: string): Promise<{ message: string }> {
    try {
      const response = await apiClient.post('/auth/password-reset/complete', {
        token,
        new_password: newPassword
      });
      return response.data;
    } catch (error) {
      console.error('Error completing password reset:', error);
      throw error;
    }
  }

  /**
   * Request MFA reset
   */
  async requestMFAReset(email: string, password: string): Promise<{ message: string }> {
    try {
      const response = await apiClient.post('/auth/mfa-reset/request', {
        email,
        password
      });
      return response.data;
    } catch (error) {
      console.error('Error requesting MFA reset:', error);
      throw error;
    }
  }

  /**
   * Complete MFA reset (returns limited-scope token for disabling MFA)
   */
  async completeMFAReset(token: string): Promise<{ token: string; user_uuid: string }> {
    try {
      const response = await apiClient.post('/auth/mfa-reset/complete', { token });
      return response.data;
    } catch (error) {
      console.error('Error completing MFA reset:', error);
      throw error;
    }
  }

  /**
   * Disable MFA with limited-scope token
   */
  async disableMFAWithToken(bearerToken: string): Promise<{ message: string }> {
    try {
      const response = await apiClient.post('/auth/mfa/disable',
        {},
        {
          headers: {
            'Authorization': `Bearer ${bearerToken}`
          }
        }
      );
      return response.data;
    } catch (error) {
      console.error('Error disabling MFA with token:', error);
      throw error;
    }
  }
}

export const authService = new AuthService();
export default authService; 