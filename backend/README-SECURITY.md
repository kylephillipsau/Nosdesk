# Nosdesk Security Documentation

## JWT Authentication

Nosdesk uses JSON Web Tokens (JWT) for authentication between the frontend and backend. This document outlines the security measures and configuration requirements.

### Environment Variables

The following environment variables must be set for secure operation:

| Variable | Description | Example |
|----------|-------------|---------|
| `JWT_SECRET` | Secret key used to sign and verify JWTs | A random, secure string (e.g., generated with `openssl rand -base64 32`) |

### How to Generate a Secure JWT Secret

To generate a secure JWT secret, run the following command:

```bash
openssl rand -base64 32
```

Add the output to your `.env` file:

```
JWT_SECRET="your-generated-secret"
```

### JWT Configuration

- **Token Expiration**: Tokens expire after 24 hours
- **Token Contents**: Contains user UUID, name, email, and role
- **Signing Algorithm**: HS256 (HMAC with SHA-256)

### Security Recommendations

1. **Avoid Hardcoding Secrets**: Never hardcode secrets in your code or commit them to version control
2. **Rotate Secrets**: Periodically rotate your JWT_SECRET in production environments
3. **Environment-Specific Secrets**: Use different secrets for development, staging, and production

## Future Security Enhancements

Consider implementing the following security enhancements:

1. **Token Refresh Mechanism**: Implement short-lived access tokens with refresh tokens
2. **Token Revocation**: Add the ability to revoke tokens before their expiration (e.g., on logout)
3. **HttpOnly Cookies**: Store tokens in HttpOnly cookies instead of localStorage for XSS protection
4. **CSRF Protection**: Add Cross-Site Request Forgery protection with custom headers or tokens

## Security Best Practices

- Always validate user input
- Use prepared statements for database queries
- Keep dependencies updated
- Implement rate limiting for authentication endpoints
- Use TLS for all communications
- Log security events and review logs regularly 