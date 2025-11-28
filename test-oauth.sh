#!/bin/bash
set -e

echo "🧪 Testing OAuth 2.1 Flow with PKCE and JWT"
echo "=========================================="
echo ""

# Step 1: OAuth Discovery
echo "✅ Step 1: Testing OAuth discovery endpoint..."
curl -s http://localhost:3000/.well-known/oauth-authorization-server | jq '.'
echo ""

# Step 2: Generate PKCE challenge
echo "✅ Step 2: Generating PKCE code verifier and challenge..."
CODE_VERIFIER=$(openssl rand -base64 32 | tr -d '=' | tr '+/' '-_')
CODE_CHALLENGE=$(echo -n "$CODE_VERIFIER" | openssl dgst -binary -sha256 | base64 | tr -d '=' | tr '+/' '-_')

echo "Code Verifier: $CODE_VERIFIER"
echo "Code Challenge: $CODE_CHALLENGE"
echo ""

# Step 3: Authorization URL
echo "✅ Step 3: Authorization Flow"
echo "Open this URL in your browser:"
echo ""
AUTH_URL="http://localhost:3000/oauth/authorize?client_id=test-client&redirect_uri=http://localhost:8080/callback&scope=mcp:tools:read%20mcp:tools:write&state=random-state-123&code_challenge=$CODE_CHALLENGE&code_challenge_method=S256"
echo "$AUTH_URL"
echo ""
echo "Instructions:"
echo "1. Copy the URL above and open it in your browser"
echo "2. Login with username: admin, password: admin123"
echo "3. You'll be redirected to http://localhost:8080/callback?code=XXX&state=random-state-123"
echo "4. Copy the 'code=XXX' part from the URL"
echo ""
read -p "Enter the authorization code: " AUTH_CODE
echo ""

# Step 4: Exchange code for token
echo "✅ Step 4: Exchanging authorization code for JWT token..."
TOKEN_RESPONSE=$(curl -s -X POST http://localhost:3000/oauth/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=authorization_code" \
  -d "code=$AUTH_CODE" \
  -d "client_id=test-client" \
  -d "redirect_uri=http://localhost:8080/callback" \
  -d "code_verifier=$CODE_VERIFIER")

echo "$TOKEN_RESPONSE" | jq '.'

ACCESS_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token')

if [ "$ACCESS_TOKEN" == "null" ] || [ -z "$ACCESS_TOKEN" ]; then
    echo "❌ Failed to get access token!"
    echo "Response: $TOKEN_RESPONSE"
    exit 1
fi

echo ""
echo "✅ Got JWT access token!"
echo ""

# Step 5: Decode JWT (just show the header and payload)
echo "✅ Step 5: JWT Token Details"
echo "You can decode this token at https://jwt.io:"
echo "$ACCESS_TOKEN"
echo ""

# Extract and decode the payload (middle part of JWT)
PAYLOAD=$(echo "$ACCESS_TOKEN" | cut -d. -f2)
# Add padding if needed
MOD=$((${#PAYLOAD} % 4))
if [ $MOD -eq 2 ]; then
    PAYLOAD="${PAYLOAD}=="
elif [ $MOD -eq 3 ]; then
    PAYLOAD="${PAYLOAD}="
fi

echo "Decoded JWT Claims:"
echo "$PAYLOAD" | base64 -d 2>/dev/null | jq '.' || echo "(Unable to decode - paste token at jwt.io)"
echo ""

# Step 6: Test MCP endpoint without token (should fail)
echo "✅ Step 6: Testing MCP endpoint WITHOUT token (should get 401)..."
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3001/mcp)
if [ "$HTTP_CODE" == "401" ]; then
    echo "✅ Correctly rejected request without token (401)"
else
    echo "❌ Expected 401, got $HTTP_CODE"
fi
echo ""

# Step 7: Test MCP endpoint with token (should work)
echo "✅ Step 7: Testing MCP endpoint WITH JWT token (should work)..."
MCP_RESPONSE=$(curl -s -w "\nHTTP_CODE:%{http_code}" \
    -H "Authorization: Bearer $ACCESS_TOKEN" \
    http://localhost:3001/mcp)

HTTP_CODE=$(echo "$MCP_RESPONSE" | grep "HTTP_CODE:" | cut -d: -f2)
BODY=$(echo "$MCP_RESPONSE" | grep -v "HTTP_CODE:")

echo "HTTP Status: $HTTP_CODE"
echo "Response: $BODY"
echo ""

if [ "$HTTP_CODE" == "200" ] || [ "$HTTP_CODE" == "404" ]; then
    echo "✅ MCP endpoint accepted JWT token! (Status: $HTTP_CODE)"
    echo ""
    echo "🎉 OAuth 2.1 + JWT authentication is working correctly!"
    echo ""
    echo "Next steps:"
    echo "1. Try connecting from Claude Desktop/Mobile"
    echo "2. Use this MCP server URL: http://localhost:3001/mcp"
    echo "3. Claude will auto-discover OAuth and walk you through authorization"
else
    echo "❌ MCP endpoint rejected token with status $HTTP_CODE"
    echo "Check server logs for details"
fi
