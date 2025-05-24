# Microsoft Entra ID Integration Setup

This document describes how to configure Microsoft Entra ID (formerly Azure AD) integration for authentication and Microsoft Graph API access.

## Environment Variables

The Microsoft Entra ID integration now uses environment variables for enhanced security. Set the following environment variables on your server:

```bash
# Required for Microsoft Authentication and Graph API
MICROSOFT_CLIENT_ID=your-application-client-id
MICROSOFT_TENANT_ID=your-tenant-id
MICROSOFT_CLIENT_SECRET=your-client-secret
MICROSOFT_REDIRECT_URI=https://your-domain.com/auth/microsoft/callback
```

## Obtaining the Required Values

### 1. Register an Application in Azure Portal

1. Go to [Azure Portal](https://portal.azure.com)
2. Navigate to **Microsoft Entra ID** (or Azure Active Directory)
3. Select **App registrations** > **New registration**
4. Fill in the registration form:
   - **Name**: Your application name (e.g., "Nosdesk Helpdesk")
   - **Supported account types**: Choose based on your needs
   - **Redirect URI**: Add `https://your-domain.com/auth/microsoft/callback`
5. Click **Register**

### 2. Get Client ID and Tenant ID

After registration:
- **Application (client) ID**: Found on the Overview page (use as `MICROSOFT_CLIENT_ID`)
- **Directory (tenant) ID**: Found on the Overview page (use as `MICROSOFT_TENANT_ID`)

### 3. Create a Client Secret

1. Go to **Certificates & secrets** > **Client secrets**
2. Click **New client secret**
3. Add a description and select expiration
4. Click **Add**
5. **IMPORTANT**: Copy the secret value immediately (use as `MICROSOFT_CLIENT_SECRET`)
   - You cannot view this secret again after leaving the page

### 4. Configure API Permissions

The application uses the following Microsoft Graph permissions:

#### Delegated Permissions (when users sign in):
- `User.Read` - Sign in and read user profile
- `Device.Read` - Read user's devices
- `Device.Read.All` - Read all devices (requires admin consent)
- `Directory.Read.All` - Read directory data (requires admin consent)
- `ProfilePhoto.Read.All` - Read all users' profile photos (requires admin consent)
- `User.Read.All` - Read all users' full profiles (requires admin consent)

#### Application Permissions (for backend service calls):
- `Directory.Read.All` - Read directory data
- `Device.Read.All` - Read all devices
- `User.Read.All` - Read all users' full profiles

To add permissions:
1. Go to **API permissions** > **Add a permission**
2. Select **Microsoft Graph**
3. Choose permission type (Delegated or Application)
4. Search and select the required permissions
5. Click **Add permissions**
6. **IMPORTANT**: Click **Grant admin consent for [your organization]**

## Setting Environment Variables

### Using .env file (Development)

Create or update your `.env` file in the backend directory:

```bash
# Microsoft Entra ID Configuration
MICROSOFT_CLIENT_ID=12345678-1234-1234-1234-123456789012
MICROSOFT_TENANT_ID=87654321-4321-4321-4321-210987654321
MICROSOFT_CLIENT_SECRET=your-secret-value-here
MICROSOFT_REDIRECT_URI=https://localhost:3000/auth/microsoft/callback
```

### Production Deployment

Set environment variables based on your deployment platform:

#### Linux/Unix Systems
```bash
export MICROSOFT_CLIENT_ID="your-client-id"
export MICROSOFT_TENANT_ID="your-tenant-id"
export MICROSOFT_CLIENT_SECRET="your-client-secret"
export MICROSOFT_REDIRECT_URI="https://your-domain.com/auth/microsoft/callback"
```

#### Docker
```dockerfile
ENV MICROSOFT_CLIENT_ID=your-client-id
ENV MICROSOFT_TENANT_ID=your-tenant-id
ENV MICROSOFT_CLIENT_SECRET=your-client-secret
ENV MICROSOFT_REDIRECT_URI=https://your-domain.com/auth/microsoft/callback
```

#### Docker Compose
```yaml
environment:
  - MICROSOFT_CLIENT_ID=your-client-id
  - MICROSOFT_TENANT_ID=your-tenant-id
  - MICROSOFT_CLIENT_SECRET=your-client-secret
  - MICROSOFT_REDIRECT_URI=https://your-domain.com/auth/microsoft/callback
```

## Verifying Configuration

1. Ensure all environment variables are set:
   ```bash
   echo $MICROSOFT_CLIENT_ID
   echo $MICROSOFT_TENANT_ID
   echo $MICROSOFT_CLIENT_SECRET
   echo $MICROSOFT_REDIRECT_URI
   ```

2. Start your backend server - it will validate that these variables are present

3. In the admin panel, navigate to Authentication Providers
   - The Microsoft provider should show as configured
   - Use the "Test Configuration" button to verify connectivity

## Security Best Practices

1. **Never commit secrets to version control**
   - Use `.gitignore` to exclude `.env` files
   - Use secret management services in production

2. **Rotate client secrets regularly**
   - Azure allows multiple active secrets
   - Update before expiration

3. **Restrict redirect URIs**
   - Only add legitimate callback URLs
   - Remove unused URLs

4. **Use least privilege principle**
   - Only request permissions your app needs
   - Remove unused permissions

## Troubleshooting

### Missing Environment Variables
If you see errors about missing configuration:
```
Microsoft configuration error: Missing environment variable: MICROSOFT_CLIENT_ID
```
Ensure all required environment variables are set and the application is restarted.

### Invalid Credentials
If authentication fails:
1. Verify the client secret hasn't expired
2. Check that the tenant ID is correct
3. Ensure the redirect URI matches exactly (including protocol and path)

### Permission Errors
If Graph API calls fail:
1. Verify admin consent was granted
2. Check that required permissions are added
3. Wait a few minutes after granting permissions (propagation delay)

## Additional Resources

- [Microsoft identity platform documentation](https://docs.microsoft.com/en-us/azure/active-directory/develop/)
- [Microsoft Graph permissions reference](https://docs.microsoft.com/en-us/graph/permissions-reference)
- [Authentication flows](https://docs.microsoft.com/en-us/azure/active-directory/develop/authentication-flows-app-scenarios) 