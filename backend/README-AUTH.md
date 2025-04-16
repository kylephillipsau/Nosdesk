# Authentication Providers in Nosdesk

This document explains how to set up external authentication providers like Microsoft Entra ID (formerly Azure AD) in Nosdesk.

## Overview

Nosdesk supports multiple authentication methods:

1. **Local Authentication** - Username and password stored in the database
2. **Microsoft Entra ID** - Single sign-on with Microsoft accounts
3. **Google Workspace** (coming soon)
4. **SAML 2.0** (coming soon)

## Setting Up Microsoft Entra ID

### Step 1: Register an app in the Microsoft Entra admin center

1. Sign in to the [Azure Portal](https://portal.azure.com)
2. Navigate to **Microsoft Entra ID** > **App registrations** > **New registration**
3. Enter a name for your application (e.g., "Nosdesk")
4. Select supported account types (usually "Accounts in this organizational directory only")
5. Add a redirect URI (Web platform): `http://localhost:8080/auth/microsoft/callback` (or your production URL)
6. Click **Register**

### Step 2: Configure app permissions

1. In your app registration, go to **API permissions**
2. Click **Add a permission**
3. Select **Microsoft Graph** > **Delegated permissions**
4. Add these permissions:
   - `User.Read` (required)
   - `email` (optional)
   - `profile` (optional)
5. Click **Add permissions**
6. Click **Grant admin consent** (if you are an admin)

### Step 3: Create a client secret

1. Go to **Certificates & secrets**
2. Click **New client secret**
3. Add a description and select expiration
4. Click **Add**
5. **IMPORTANT**: Copy the secret value immediately; you won't be able to see it again

### Step 4: Collect required information

You'll need:
- **Application (client) ID**: Found on the app overview page
- **Directory (tenant) ID**: Found on the app overview page
- **Client Secret**: The value you copied in step 3
- **Redirect URI**: The URL you configured in step 1

### Step 5: Configure Nosdesk

1. Log in to Nosdesk as an admin
2. Go to Administration > Authentication Providers
3. Click on Microsoft Entra ID provider
4. Enter the following details:
   - Application (client) ID
   - Directory (tenant) ID
   - Client Secret
   - Redirect URI (should match what you configured in Azure)
5. Enable the provider and click Save

## Security Considerations

- Always use HTTPS in production
- Store client secrets securely
- Regularly rotate client secrets
- Review user permissions and access regularly

## Troubleshooting

Common issues:

1. **Redirect URI mismatch**: The redirect URI in Nosdesk must exactly match the one in your Azure app registration
2. **Missing permissions**: Make sure you've granted the necessary permissions
3. **Invalid client secret**: Client secrets expire; check if you need to create a new one
4. **Tenant restrictions**: Some organizations restrict app registrations

## Additional Resources

- [Microsoft Entra ID documentation](https://learn.microsoft.com/en-us/entra/identity/app-provisioning/user-provisioning)
- [OAuth 2.0 and OpenID Connect protocols](https://learn.microsoft.com/en-us/entra/identity/standards/index) 